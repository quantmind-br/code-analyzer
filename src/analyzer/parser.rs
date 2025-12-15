use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tree_sitter::{Node, Tree};

use super::language::{LanguageManager, NodeKindMapper, SupportedLanguage};
use crate::error::{AnalyzerError, ParseWarning, ParseWarningLocation, Result};

/// Complete result from file analysis including warnings
#[derive(Debug)]
pub struct FileAnalysisResult {
    pub analysis: FileAnalysis,
    pub warnings: Vec<ParseWarning>,
}

/// Analysis result for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAnalysis {
    pub path: PathBuf,
    pub language: String,
    pub lines_of_code: usize,
    pub blank_lines: usize,
    pub comment_lines: usize,
    pub functions: usize,
    pub methods: usize,
    pub classes: usize,
    pub cyclomatic_complexity: usize,
    pub complexity_score: f64,
}

impl FileAnalysis {
    /// Get total lines (code + blank + comments)
    pub fn total_lines(&self) -> usize {
        self.lines_of_code + self.blank_lines + self.comment_lines
    }

    /// Calculate complexity score using cyclomatic complexity as primary factor
    ///
    /// Formula: loc_factor + cyclomatic_factor + structure_factor
    ///
    /// The weights are calibrated so that:
    /// - A file with 500 LOC, CC=15, 10 functions → score ≈ 10.0 (threshold)
    /// - Cyclomatic complexity has the highest weight (industry best practice)
    /// - LOC contributes proportionally but with diminishing returns
    /// - Structure factor accounts for module organization
    pub fn calculate_complexity(&mut self) {
        // LOC factor: normalized with diminishing returns for very large files
        // 500 LOC → 2.5 points (at threshold)
        let loc_score = (self.lines_of_code as f64 / 200.0).min(5.0);

        // Cyclomatic complexity is the primary factor (per NIST/McCabe)
        // CC of 1-10: low, 11-20: moderate, 21-50: high, 50+: very high
        // CC=15 → 6.0 points (at threshold)
        let cc_score = self.cyclomatic_complexity as f64 * 0.4;

        // Structure factor: rewards modular design with many small functions
        // 10 functions → ~0.95 points, 25 functions → ~1.5 points
        // Classes add minor weight
        let structure_score =
            (self.functions as f64).sqrt() * 0.3 + (self.classes as f64).sqrt() * 0.15;

        self.complexity_score = loc_score + cc_score + structure_score;
    }
}

/// Complete analysis report structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnalysisReport {
    pub files: Vec<FileAnalysis>,
    pub summary: ProjectSummary,
    pub config: AnalysisConfig,
    pub generated_at: DateTime<Utc>,
    /// Non-fatal warnings encountered during parsing
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<crate::error::ParseWarning>,
}

/// Project-wide summary statistics
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectSummary {
    pub total_files: usize,
    pub total_lines: usize,
    pub total_functions: usize,
    pub total_methods: usize,
    pub total_classes: usize,
    pub language_breakdown: HashMap<String, LanguageStats>,
    pub largest_files: Vec<FileAnalysis>,
    pub most_complex_files: Vec<FileAnalysis>,
}

/// Statistics for a specific language
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LanguageStats {
    pub file_count: usize,
    pub total_lines: usize,
    pub avg_functions_per_file: f64,
    pub avg_methods_per_file: f64,
    pub avg_classes_per_file: f64,
}

/// Configuration used for the analysis
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnalysisConfig {
    pub target_path: PathBuf,
    pub languages: Vec<String>,
    pub min_lines: usize,
    pub max_lines: Option<usize>,
    pub include_hidden: bool,
    pub max_file_size_mb: usize,
}

/// Reason why a file is flagged for refactoring
#[derive(Debug, Clone, PartialEq)]
pub enum RefactoringReason {
    /// Complexity score is too high (score >= 10.0)
    HighComplexityScore(f64),
    /// Cyclomatic complexity is too high (CC >= 20)
    HighCyclomaticComplexity(usize),
    /// File has too many lines (lines >= 500)
    LargeFile(usize),
    /// File has too many functions (functions >= 20)
    TooManyFunctions(usize),
}

impl RefactoringReason {
    /// Get a short description of the reason
    pub fn short_description(&self) -> &'static str {
        match self {
            RefactoringReason::HighComplexityScore(_) => "High Score",
            RefactoringReason::HighCyclomaticComplexity(_) => "High CC",
            RefactoringReason::LargeFile(_) => "Large file",
            RefactoringReason::TooManyFunctions(_) => "Many funcs",
        }
    }
}

/// A file identified as a refactoring candidate
#[derive(Debug, Clone)]
pub struct RefactoringCandidate {
    pub file: FileAnalysis,
    pub reasons: Vec<RefactoringReason>,
}

impl RefactoringCandidate {
    /// Get a comma-separated string of all reasons
    pub fn reasons_string(&self) -> String {
        self.reasons
            .iter()
            .map(|r| r.short_description())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Configurable thresholds for identifying refactoring candidates
///
/// Default values are based on industry standards and best practices:
/// - Cyclomatic Complexity: 15 (NIST 500-235 suggests 10-15 for experienced teams)
/// - Lines of Code: 500 (pragmatic threshold for real-world projects)
/// - Functions per file: 25 (allows well-organized modules with small, cohesive functions)
/// - Complexity Score: 10.0 (composite threshold aligned with relaxed metrics)
///
/// References:
/// - NIST SP 500-235: https://www.mccabe.com/nist/nist_pub.php
/// - NASA SLS uses CC threshold of 20
/// - SonarQube uses Cognitive Complexity threshold of 15
#[derive(Debug, Clone)]
pub struct RefactoringThresholds {
    /// Complexity score threshold (default: 10.0)
    pub max_complexity_score: f64,
    /// Cyclomatic complexity threshold (default: 15, per NIST guidance for mature teams)
    pub max_cyclomatic_complexity: usize,
    /// Lines of code threshold (default: 500)
    pub max_lines_of_code: usize,
    /// Functions per file threshold (default: 25)
    pub max_functions: usize,
}

impl Default for RefactoringThresholds {
    fn default() -> Self {
        Self {
            max_complexity_score: 10.0,
            max_cyclomatic_complexity: 15,
            max_lines_of_code: 500,
            max_functions: 25,
        }
    }
}

impl RefactoringThresholds {
    /// Create thresholds from CLI arguments, using defaults for unspecified values
    pub fn from_cli(args: &crate::cli::CliArgs) -> Self {
        Self {
            max_complexity_score: args.max_complexity_score.unwrap_or(10.0),
            max_cyclomatic_complexity: args.max_cc.unwrap_or(15),
            max_lines_of_code: args.max_loc.unwrap_or(500),
            max_functions: args.max_functions_per_file.unwrap_or(25),
        }
    }
}

/// Identify files that are candidates for refactoring based on configurable thresholds
pub fn identify_refactoring_candidates(
    files: &[FileAnalysis],
    thresholds: &RefactoringThresholds,
) -> Vec<RefactoringCandidate> {
    let mut candidates = Vec::new();

    for file in files {
        let mut reasons = Vec::new();

        // Check complexity score against threshold
        if file.complexity_score >= thresholds.max_complexity_score {
            reasons.push(RefactoringReason::HighComplexityScore(
                file.complexity_score,
            ));
        }

        // Check cyclomatic complexity against threshold
        if file.cyclomatic_complexity >= thresholds.max_cyclomatic_complexity {
            reasons.push(RefactoringReason::HighCyclomaticComplexity(
                file.cyclomatic_complexity,
            ));
        }

        // Check file size against threshold
        if file.lines_of_code >= thresholds.max_lines_of_code {
            reasons.push(RefactoringReason::LargeFile(file.lines_of_code));
        }

        // Check function count against threshold
        if file.functions >= thresholds.max_functions {
            reasons.push(RefactoringReason::TooManyFunctions(file.functions));
        }

        // Only include if at least one reason
        if !reasons.is_empty() {
            candidates.push(RefactoringCandidate {
                file: file.clone(),
                reasons,
            });
        }
    }

    // Sort by complexity score (highest first)
    candidates.sort_by(|a, b| {
        b.file
            .complexity_score
            .partial_cmp(&a.file.complexity_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    candidates
}

/// Core file parser using tree-sitter
#[derive(Clone)]
pub struct FileParser {
    language_manager: LanguageManager,
    max_file_size_bytes: u64,
}

impl FileParser {
    /// Create a new file parser
    pub fn new(language_manager: LanguageManager, max_file_size_mb: usize) -> Self {
        Self {
            language_manager,
            max_file_size_bytes: max_file_size_mb as u64 * 1024 * 1024,
        }
    }

    /// Parse a single file and extract metrics
    pub fn parse_file_metrics<P: AsRef<Path>>(&mut self, path: P) -> Result<FileAnalysis> {
        let result = self.parse_file_with_warnings(path)?;
        Ok(result.analysis)
    }

    /// Parse a single file and extract metrics, including any warnings
    pub fn parse_file_with_warnings<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<FileAnalysisResult> {
        let path = path.as_ref();
        let mut warnings = Vec::new();

        // Detect language
        let language = self.language_manager.detect_language(path).ok_or_else(|| {
            AnalyzerError::unsupported_language(
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("unknown"),
            )
        })?;

        // Check file size before reading
        let metadata = fs::metadata(path)?;
        if metadata.len() > self.max_file_size_bytes {
            return Err(AnalyzerError::validation_error(format!(
                "File too large: {} bytes",
                metadata.len()
            )));
        }

        // Read file contents
        let source_code = fs::read(path)?;

        // Validate UTF-8 encoding
        let source_text = match std::str::from_utf8(&source_code) {
            Ok(text) => text,
            Err(e) => {
                warnings.push(ParseWarning::encoding_error(
                    path,
                    format!("Invalid UTF-8 encoding: {e}"),
                ));
                return Err(AnalyzerError::parse_error(format!(
                    "Invalid UTF-8 encoding: {e}"
                )));
            }
        };

        // Parse with tree-sitter
        let parser = self.language_manager.get_parser(language)?;
        let parse_result = parse_file_safely(parser, &source_code, source_text, language, path)?;

        // Collect any parsing warnings
        warnings.extend(parse_result.warnings);

        let tree = parse_result.tree;

        // Count lines (basic count for blank lines, AST for comments)
        let line_counts = count_lines(source_text);

        // Use AST-based comment counting for accuracy (falls back to heuristic if no tree)
        let comment_lines = if tree.is_some() {
            count_comment_lines_ast(&tree, &language)
        } else {
            line_counts.comments
        };

        // Recalculate code lines based on AST comment count
        let total_non_blank = source_text.lines().filter(|l| !l.trim().is_empty()).count();
        let lines_of_code = total_non_blank.saturating_sub(comment_lines);

        // Extract AST metrics
        let functions = if let Some(ref tree) = tree {
            count_functions(&tree.root_node(), &source_code, &language)
        } else {
            0
        };

        let methods = if let Some(ref tree) = tree {
            count_methods(&tree.root_node(), &language)
        } else {
            0
        };

        let classes = if let Some(ref tree) = tree {
            count_classes(&tree.root_node(), &source_code, &language)
        } else {
            0
        };

        // Calculate cyclomatic complexity
        let cyclomatic_complexity = calculate_cyclomatic_complexity(&tree, &language);

        let mut analysis = FileAnalysis {
            path: path.to_path_buf(),
            language: language.to_string(),
            lines_of_code,
            blank_lines: line_counts.blank,
            comment_lines,
            functions,
            methods,
            classes,
            cyclomatic_complexity,
            complexity_score: 0.0,
        };

        // Calculate complexity score (uses cyclomatic_complexity)
        analysis.calculate_complexity();

        Ok(FileAnalysisResult { analysis, warnings })
    }

    /// Check if a file can be parsed (size and language support)
    pub fn can_parse<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = path.as_ref();

        // Check if language is supported
        if !self.language_manager.is_supported_file(path) {
            return false;
        }

        // Check file size
        if let Ok(metadata) = fs::metadata(path) {
            metadata.len() <= self.max_file_size_bytes
        } else {
            false
        }
    }

    /// Get statistics about the language manager
    pub fn language_stats(&self) -> HashMap<SupportedLanguage, usize> {
        self.language_manager.parser_stats()
    }

    /// Get the maximum file size in bytes
    pub fn max_file_size_bytes(&self) -> u64 {
        self.max_file_size_bytes
    }
}

/// Line counting results (heuristic-based, used as fallback when AST unavailable)
#[derive(Debug)]
#[allow(dead_code)]
struct LineCounts {
    code: usize,
    blank: usize,
    comments: usize,
}

/// Count different types of lines in source code
fn count_lines(source: &str) -> LineCounts {
    let mut code_lines = 0;
    let mut blank_lines = 0;
    let mut comment_lines = 0;

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            blank_lines += 1;
        } else if is_comment_line(trimmed) {
            comment_lines += 1;
        } else {
            code_lines += 1;
        }
    }

    LineCounts {
        code: code_lines,
        blank: blank_lines,
        comments: comment_lines,
    }
}

/// Basic heuristic to detect comment lines
fn is_comment_line(line: &str) -> bool {
    line.starts_with("//")  // C-style, Rust, JavaScript, etc.
        || line.starts_with('#')   // Python, shell, etc.
        || line.starts_with("/*")  // C-style block comment start
        || line.starts_with("*/")  // C-style block comment end
        || line.starts_with("*")   // C-style block comment continuation
        || line.starts_with("<!--") // HTML comment
        || line.starts_with("-->") // HTML comment end
}

/// Parse result including tree and any warnings
pub struct ParseResult {
    pub tree: Option<Tree>,
    pub warnings: Vec<ParseWarning>,
}

/// Safely parse a file with error recovery and warning collection
fn parse_file_safely(
    parser: &mut tree_sitter::Parser,
    source: &[u8],
    source_text: &str,
    language: SupportedLanguage,
    file_path: &Path,
) -> Result<ParseResult> {
    let sanitized = sanitize_for_tree_sitter(source_text, language);
    let original_tree = match parser.parse(source, None) {
        Some(tree) => tree,
        None => {
            return Err(AnalyzerError::tree_sitter_error(
                "Failed to parse file - tree-sitter returned None",
            ));
        }
    };

    // If the original source parses cleanly, keep it as-is.
    if !original_tree.root_node().has_error() {
        return Ok(ParseResult {
            tree: Some(original_tree),
            warnings: Vec::new(),
        });
    }

    // If there are parse errors, try a best-effort sanitization for known grammar gaps.
    if let Cow::Owned(sanitized_text) = sanitized {
        if let Some(sanitized_tree) = parser.parse(sanitized_text.as_bytes(), None) {
            if !sanitized_tree.root_node().has_error() {
                return Ok(ParseResult {
                    tree: Some(sanitized_tree),
                    warnings: Vec::new(),
                });
            }
        }
    }

    // Still has errors: keep the original tree and surface warning details.
    match Some(original_tree) {
        Some(tree) => {
            let mut warnings = Vec::new();

            if tree.root_node().has_error() {
                let locations = collect_parse_error_locations(&tree, source_text, 5);
                let message = match locations.first() {
                    Some(first) => format!(
                        "Parse errors detected near {}:{} ({})",
                        first.line, first.column, first.kind
                    ),
                    None => "Parse errors detected, results may be incomplete".to_string(),
                };

                let mut warning = ParseWarning::syntax_error(file_path, message);
                warning.locations = locations;
                warnings.push(warning);
            }

            Ok(ParseResult {
                tree: Some(tree),
                warnings,
            })
        }
        None => unreachable!("tree was already constructed"),
    }
}

fn sanitize_for_tree_sitter(source_text: &str, language: SupportedLanguage) -> Cow<'_, str> {
    match language {
        SupportedLanguage::TypeScript => {
            // tree-sitter-typescript doesn't reliably parse TS 5.0 `export type * from ...` yet.
            if source_text.contains("export type * from") {
                Cow::Owned(source_text.replace("export type * from", "export * from"))
            } else {
                Cow::Borrowed(source_text)
            }
        }
        SupportedLanguage::Tsx => {
            let mut result = Cow::Borrowed(source_text);

            if result.contains("export type * from") {
                result = Cow::Owned(result.replace("export type * from", "export * from"));
            }

            // tree-sitter's TSX grammar treats JSX text as XML, so `&` must be escaped.
            // Many TSX codebases include raw `&`/`&&` in JSX text nodes; sanitize for parsing only.
            if result.contains('&') && result.contains('<') {
                let escaped = escape_ampersands_in_jsx_text(&result);
                if escaped != *result {
                    result = Cow::Owned(escaped);
                }
            }

            result
        }
        _ => Cow::Borrowed(source_text),
    }
}

fn escape_ampersands_in_jsx_text(source: &str) -> String {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum State {
        Normal,
        InTag,
        InText,
        InExpr,
    }

    fn is_ident_char(ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'
    }

    fn prev_non_ws_char(chars: &[char], mut i: usize) -> Option<char> {
        while i > 0 {
            i -= 1;
            let ch = chars[i];
            if !ch.is_whitespace() {
                return Some(ch);
            }
        }
        None
    }

    fn looks_like_entity(chars: &[char], i: usize) -> bool {
        if i >= chars.len() || chars[i] != '&' {
            return false;
        }
        let next = match chars.get(i + 1) {
            Some(c) => *c,
            None => return false,
        };

        if next == '#' {
            // &#123; or &#x1F600;
            let mut j = i + 2;
            if matches!(chars.get(j), Some('x') | Some('X')) {
                j += 1;
                let mut has_hex = false;
                while let Some(c) = chars.get(j) {
                    if c.is_ascii_hexdigit() {
                        has_hex = true;
                        j += 1;
                        continue;
                    }
                    break;
                }
                return has_hex && matches!(chars.get(j), Some(';'));
            }

            let mut has_digit = false;
            while let Some(c) = chars.get(j) {
                if c.is_ascii_digit() {
                    has_digit = true;
                    j += 1;
                    continue;
                }
                break;
            }
            return has_digit && matches!(chars.get(j), Some(';'));
        }

        if !next.is_ascii_alphabetic() {
            return false;
        }

        let mut j = i + 2;
        while let Some(c) = chars.get(j) {
            if c.is_ascii_alphanumeric() {
                j += 1;
                continue;
            }
            break;
        }

        matches!(chars.get(j), Some(';'))
    }

    fn is_jsx_tag_start_in_text(chars: &[char], i: usize) -> bool {
        if chars.get(i) != Some(&'<') {
            return false;
        }
        let next = match chars.get(i + 1) {
            Some(c) => *c,
            None => return false,
        };

        next.is_ascii_alphabetic() || next == '/' || next == '>'
    }

    fn is_probably_jsx_tag_start_in_normal(chars: &[char], i: usize) -> bool {
        if !is_jsx_tag_start_in_text(chars, i) {
            return false;
        }

        !matches!(prev_non_ws_char(chars, i), Some(prev) if is_ident_char(prev) || prev == '.' || prev == ')' || prev == ']')
    }

    let chars: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());

    let mut state = State::Normal;
    let mut jsx_depth: usize = 0;
    let mut jsx_entry_stack: Vec<usize> = Vec::new();
    let mut tag_quote: Option<char> = None;
    let mut tag_brace_depth: usize = 0;
    let mut expr_depth: usize = 0;

    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];

        match state {
            State::Normal => {
                if is_probably_jsx_tag_start_in_normal(&chars, i) {
                    state = State::InTag;
                    tag_quote = None;
                    tag_brace_depth = 0;
                }
                out.push(ch);
                i += 1;
            }
            State::InTag => {
                out.push(ch);

                if let Some(q) = tag_quote {
                    if ch == q {
                        tag_quote = None;
                    }
                    i += 1;
                    continue;
                }

                match ch {
                    '"' | '\'' => tag_quote = Some(ch),
                    '{' => tag_brace_depth += 1,
                    '}' => tag_brace_depth = tag_brace_depth.saturating_sub(1),
                    '>' if tag_brace_depth == 0 => {
                        // Determine if this tag changes depth.
                        let mut j = i;
                        while j > 0 && chars[j].is_whitespace() {
                            j -= 1;
                        }
                        let mut k = i;
                        while k > 0 {
                            k -= 1;
                            if chars[k] == '<' {
                                break;
                            }
                        }
                        let is_closing = chars.get(k + 1) == Some(&'/');
                        let self_closing = !is_closing && chars.get(j) == Some(&'/');

                        if self_closing {
                            // no-op
                        } else if is_closing {
                            jsx_depth = jsx_depth.saturating_sub(1);
                        } else {
                            jsx_depth += 1;
                        }

                        if matches!(jsx_entry_stack.last(), Some(&entry) if jsx_depth == entry) {
                            jsx_entry_stack.pop();
                            state = State::InExpr;
                        } else {
                            state = if jsx_depth == 0 {
                                State::Normal
                            } else {
                                State::InText
                            };
                        }
                    }
                    _ => {}
                }

                i += 1;
            }
            State::InText => match ch {
                '{' => {
                    state = State::InExpr;
                    expr_depth = 1;
                    out.push(ch);
                    i += 1;
                }
                '<' if is_jsx_tag_start_in_text(&chars, i) => {
                    state = State::InTag;
                    tag_quote = None;
                    tag_brace_depth = 0;
                    out.push(ch);
                    i += 1;
                }
                '&' => {
                    if looks_like_entity(&chars, i) {
                        out.push('&');
                    } else {
                        out.push_str("&amp;");
                    }
                    i += 1;
                }
                _ => {
                    out.push(ch);
                    i += 1;
                }
            },
            State::InExpr => {
                if ch == '<' && is_probably_jsx_tag_start_in_normal(&chars, i) {
                    jsx_entry_stack.push(jsx_depth);
                    state = State::InTag;
                    tag_quote = None;
                    tag_brace_depth = 0;
                    out.push(ch);
                    i += 1;
                    continue;
                }

                out.push(ch);
                match ch {
                    '{' => expr_depth += 1,
                    '}' => {
                        expr_depth = expr_depth.saturating_sub(1);
                        if expr_depth == 0 {
                            state = State::InText;
                        }
                    }
                    _ => {}
                }
                i += 1;
            }
        }
    }

    out
}

fn collect_parse_error_locations(
    tree: &Tree,
    source_text: &str,
    max_locations: usize,
) -> Vec<ParseWarningLocation> {
    let mut locations = Vec::new();
    let lines: Vec<&str> = source_text.lines().collect();

    let mut cursor = tree.root_node().walk();
    loop {
        let node = cursor.node();
        if node.is_error() || node.is_missing() {
            let pos = node.start_position();
            let row = pos.row;
            let col = pos.column;
            let snippet = lines.get(row).map(|l| {
                let mut s = (*l).trim_end().to_string();
                if s.len() > 200 {
                    s.truncate(200);
                }
                s
            });

            locations.push(ParseWarningLocation {
                line: row + 1,
                column: col + 1,
                kind: node.kind().to_string(),
                snippet,
            });

            if locations.len() >= max_locations {
                break;
            }
        }

        if cursor.goto_first_child() {
            continue;
        }

        loop {
            if cursor.goto_next_sibling() {
                break;
            }
            if !cursor.goto_parent() {
                return locations;
            }
        }
    }

    locations
}

/// Generic iterative AST traversal using TreeCursor (stack-safe)
fn count_nodes_iterative<F>(root: &Node, is_target: F) -> usize
where
    F: Fn(&str) -> bool,
{
    let mut count = 0;
    let mut cursor = root.walk();

    loop {
        // Check current node
        if is_target(cursor.node().kind()) {
            count += 1;
        }

        // Try to descend to first child
        if cursor.goto_first_child() {
            continue;
        }

        // No children, try siblings or go up
        loop {
            if cursor.goto_next_sibling() {
                break;
            }

            if !cursor.goto_parent() {
                return count; // Reached root, traversal complete
            }
        }
    }
}

/// Count function declarations in an AST tree (iterative, stack-safe)
fn count_functions(node: &Node, _source: &[u8], language: &SupportedLanguage) -> usize {
    count_nodes_iterative(node, |kind| language.is_function_node(kind))
}

/// Count class/struct declarations in an AST tree (iterative, stack-safe)
fn count_classes(node: &Node, _source: &[u8], language: &SupportedLanguage) -> usize {
    count_nodes_iterative(node, |kind| language.is_class_node(kind))
}

/// Count control flow nodes for cyclomatic complexity (iterative, stack-safe)
fn count_control_flow(node: &Node, language: &SupportedLanguage) -> usize {
    count_nodes_iterative(node, |kind| language.is_control_flow_node(kind))
}

/// Count method declarations in an AST tree (iterative, stack-safe)
fn count_methods(node: &Node, language: &SupportedLanguage) -> usize {
    count_nodes_iterative(node, |kind| language.is_method_node(kind))
}

/// Calculate cyclomatic complexity: 1 + number of decision points
/// This follows the formula: M = E - N + 2P, simplified for single-component graphs
fn calculate_cyclomatic_complexity(tree: &Option<Tree>, language: &SupportedLanguage) -> usize {
    match tree {
        Some(tree) => {
            // Base complexity is 1 (for the main path)
            // Each decision point adds 1 to the complexity
            1 + count_control_flow(&tree.root_node(), language)
        }
        None => 1, // Minimum complexity for unparseable files
    }
}

/// Count comment lines using AST - more accurate than heuristic
/// Returns the number of unique lines that contain comments
fn count_comment_lines_ast(tree: &Option<Tree>, language: &SupportedLanguage) -> usize {
    use std::collections::HashSet;

    match tree {
        Some(tree) => {
            let mut comment_lines = HashSet::new();
            let mut cursor = tree.root_node().walk();

            loop {
                let node = cursor.node();

                // Check if this is a comment node
                if language.is_comment_node(node.kind()) {
                    // Add all lines this comment spans
                    let start_line = node.start_position().row;
                    let end_line = node.end_position().row;
                    for line in start_line..=end_line {
                        comment_lines.insert(line);
                    }
                }

                // Navigate tree iteratively
                if cursor.goto_first_child() {
                    continue;
                }

                loop {
                    if cursor.goto_next_sibling() {
                        break;
                    }
                    if !cursor.goto_parent() {
                        return comment_lines.len();
                    }
                }
            }
        }
        None => 0,
    }
}

/// Create a project summary from analysis results
pub fn create_project_summary(files: &[FileAnalysis]) -> ProjectSummary {
    let total_files = files.len();
    let total_lines = files.iter().map(|f| f.total_lines()).sum();
    let total_functions = files.iter().map(|f| f.functions).sum();
    let total_methods = files.iter().map(|f| f.methods).sum();
    let total_classes = files.iter().map(|f| f.classes).sum();

    // Calculate language breakdown
    let mut language_breakdown = HashMap::new();
    for file in files {
        let entry = language_breakdown
            .entry(file.language.clone())
            .or_insert_with(|| LanguageStats {
                file_count: 0,
                total_lines: 0,
                avg_functions_per_file: 0.0,
                avg_methods_per_file: 0.0,
                avg_classes_per_file: 0.0,
            });

        entry.file_count += 1;
        entry.total_lines += file.total_lines();
    }

    // Calculate averages for each language
    for (lang, stats) in language_breakdown.iter_mut() {
        let lang_files: Vec<_> = files.iter().filter(|f| f.language == *lang).collect();
        let total_functions: usize = lang_files.iter().map(|f| f.functions).sum();
        let total_methods: usize = lang_files.iter().map(|f| f.methods).sum();
        let total_classes: usize = lang_files.iter().map(|f| f.classes).sum();

        stats.avg_functions_per_file = if stats.file_count > 0 {
            total_functions as f64 / stats.file_count as f64
        } else {
            0.0
        };

        stats.avg_methods_per_file = if stats.file_count > 0 {
            total_methods as f64 / stats.file_count as f64
        } else {
            0.0
        };

        stats.avg_classes_per_file = if stats.file_count > 0 {
            total_classes as f64 / stats.file_count as f64
        } else {
            0.0
        };
    }

    // Get largest files (top 10 by total lines)
    let mut largest_files = files.to_vec();
    largest_files.sort_by_key(|f| std::cmp::Reverse(f.total_lines()));
    largest_files.truncate(10);

    // Get most complex files (top 10 by complexity score)
    let mut most_complex_files = files.to_vec();
    most_complex_files.sort_by(|a, b| b.complexity_score.partial_cmp(&a.complexity_score).unwrap());
    most_complex_files.truncate(10);

    ProjectSummary {
        total_files,
        total_lines,
        total_functions,
        total_methods,
        total_classes,
        language_breakdown,
        largest_files,
        most_complex_files,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_counting() {
        let source = r#"
// This is a comment
fn main() {
    println!("Hello, world!");
    
    // Another comment
}

/* Block comment */
        "#;

        let counts = count_lines(source);
        assert!(counts.code > 0);
        assert!(counts.blank > 0);
        assert!(counts.comments > 0);
    }

    #[test]
    fn test_is_comment_line() {
        assert!(is_comment_line("// Single line comment"));
        assert!(is_comment_line("# Python comment"));
        assert!(is_comment_line("/* Block comment */"));
        assert!(!is_comment_line("let x = 5;"));
        assert!(!is_comment_line("    code_with_spaces"));
    }

    #[test]
    fn test_file_analysis_complexity() {
        let mut analysis = FileAnalysis {
            path: PathBuf::from("test.rs"),
            language: "rust".to_string(),
            lines_of_code: 100,
            blank_lines: 10,
            comment_lines: 20,
            functions: 5,
            methods: 3,
            classes: 2,
            cyclomatic_complexity: 10,
            complexity_score: 0.0,
        };

        analysis.calculate_complexity();
        assert!(analysis.complexity_score > 0.0);
    }

    #[test]
    fn test_create_project_summary() {
        let files = vec![
            FileAnalysis {
                path: PathBuf::from("test1.rs"),
                language: "rust".to_string(),
                lines_of_code: 100,
                blank_lines: 10,
                comment_lines: 5,
                functions: 3,
                methods: 2,
                classes: 1,
                cyclomatic_complexity: 5,
                complexity_score: 2.5,
            },
            FileAnalysis {
                path: PathBuf::from("test2.rs"),
                language: "rust".to_string(),
                lines_of_code: 200,
                blank_lines: 20,
                comment_lines: 10,
                functions: 5,
                methods: 4,
                classes: 2,
                cyclomatic_complexity: 8,
                complexity_score: 4.0,
            },
        ];

        let summary = create_project_summary(&files);
        assert_eq!(summary.total_files, 2);
        assert_eq!(summary.total_lines, 345); // 100+10+5 + 200+20+10
        assert_eq!(summary.total_functions, 8);
        assert_eq!(summary.total_classes, 3);
        assert!(summary.language_breakdown.contains_key("rust"));

        let rust_stats = &summary.language_breakdown["rust"];
        assert_eq!(rust_stats.file_count, 2);
        assert_eq!(rust_stats.avg_functions_per_file, 4.0); // 8/2
    }

    #[test]
    fn test_sanitize_typescript_export_type_star() {
        let input = r#"export type * from "../documents/types";"#;
        let sanitized = sanitize_for_tree_sitter(input, SupportedLanguage::TypeScript);
        assert!(sanitized.contains("export * from"));
        assert!(!sanitized.contains("export type * from"));
    }

    #[test]
    fn test_sanitize_tsx_escapes_ampersand_in_jsx_text() {
        let input = r#"export const C = () => (<div>Effects & Animations</div>);"#;
        let sanitized = sanitize_for_tree_sitter(input, SupportedLanguage::Tsx);
        assert!(sanitized.contains("Effects &amp; Animations"));
    }

    #[test]
    fn test_sanitize_tsx_escapes_ampersand_in_jsx_inside_expression() {
        let input = r#"
export const C = () => (
  <div>
    {cond ? (<p>a & b</p>) : (<p>c && d</p>)}
  </div>
);
"#;
        let sanitized = sanitize_for_tree_sitter(input, SupportedLanguage::Tsx);
        assert!(sanitized.contains("<p>a &amp; b</p>"));
        assert!(sanitized.contains("<p>c &amp;&amp; d</p>"));
    }

    #[test]
    fn test_sanitize_tsx_does_not_escape_type_intersection() {
        let input = r#"
export const X = React.forwardRef<
  React.ElementRef<typeof DialogPrimitive.Content>,
  React.ComponentPropsWithoutRef<typeof DialogPrimitive.Content> & {
    showCloseButton?: boolean;
  }
>(() => (<div>a & b</div>));
"#;
        let sanitized = sanitize_for_tree_sitter(input, SupportedLanguage::Tsx);
        assert!(sanitized.contains("ComponentPropsWithoutRef<typeof DialogPrimitive.Content> & {"));
        assert!(sanitized.contains("<div>a &amp; b</div>"));
    }

    #[test]
    fn test_parse_file_with_warnings_encoding_error() {
        use crate::analyzer::language::LanguageManager;
        use tempfile::NamedTempFile;

        let language_manager = LanguageManager::new();
        let mut parser = FileParser::new(language_manager, 10);

        // Create a temporary file with invalid UTF-8 bytes
        let temp_file = NamedTempFile::new().unwrap();
        let invalid_utf8 = vec![0xFF, 0xFE, 0xFD]; // Invalid UTF-8 sequence
        std::fs::write(temp_file.path(), invalid_utf8).unwrap();

        // This should return an error due to encoding issues
        let result = parser.parse_file_with_warnings(temp_file.path());
        assert!(result.is_err());

        let _ = temp_file.close();
    }

    #[test]
    fn test_file_analysis_result_structure() {
        let file = FileAnalysis {
            path: PathBuf::from("test.rs"),
            language: "rust".to_string(),
            lines_of_code: 50,
            blank_lines: 5,
            comment_lines: 10,
            functions: 3,
            methods: 2,
            classes: 1,
            cyclomatic_complexity: 5,
            complexity_score: 2.5,
        };

        let result = FileAnalysisResult {
            analysis: file,
            warnings: vec![],
        };

        assert_eq!(result.analysis.lines_of_code, 50);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_parse_result_structure() {
        // This tests the ParseResult structure used internally
        let parse_result = ParseResult {
            tree: None,
            warnings: vec![],
        };

        assert!(parse_result.tree.is_none());
        assert!(parse_result.warnings.is_empty());
    }

    #[test]
    fn test_count_nodes_iterative_empty_tree() {
        use crate::analyzer::language::SupportedLanguage;

        let source = "fn main() {}";
        let _language = SupportedLanguage::Rust;

        // Create a minimal tree for testing
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();

        // Test with a non-matching kind (should return 0)
        let count = count_nodes_iterative(&root, |kind| kind == "nonexistent_node");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_calculate_cyclomatic_complexity_none_tree() {
        use crate::analyzer::language::SupportedLanguage;

        let language = SupportedLanguage::Rust;
        let complexity = calculate_cyclomatic_complexity(&None, &language);

        // Should return minimum complexity of 1
        assert_eq!(complexity, 1);
    }
}
