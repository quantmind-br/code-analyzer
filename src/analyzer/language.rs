use crate::error::{AnalyzerError, Result};
use std::collections::HashMap;
use std::path::Path;
use tree_sitter::{Language, Parser};

/// Supported programming languages with their tree-sitter grammars
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedLanguage {
    Rust,
    JavaScript,
    TypeScript,
    Tsx,
    Python,
    Java,
    C,
    Cpp,
    Go,
}

impl SupportedLanguage {
    /// Get the tree-sitter language grammar for this language
    pub fn get_grammar(&self) -> Language {
        match self {
            SupportedLanguage::Rust => tree_sitter_rust::LANGUAGE.into(),
            SupportedLanguage::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            SupportedLanguage::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            SupportedLanguage::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
            SupportedLanguage::Python => tree_sitter_python::LANGUAGE.into(),
            SupportedLanguage::Java => tree_sitter_java::LANGUAGE.into(),
            SupportedLanguage::C => tree_sitter_c::LANGUAGE.into(),
            SupportedLanguage::Cpp => tree_sitter_cpp::LANGUAGE.into(),
            SupportedLanguage::Go => tree_sitter_go::LANGUAGE.into(),
        }
    }

    /// Get the human-readable name of the language
    pub fn name(&self) -> &'static str {
        match self {
            SupportedLanguage::Rust => "rust",
            SupportedLanguage::JavaScript => "javascript",
            SupportedLanguage::TypeScript => "typescript",
            SupportedLanguage::Tsx => "typescript", // TSX is grouped with TypeScript in reports
            SupportedLanguage::Python => "python",
            SupportedLanguage::Java => "java",
            SupportedLanguage::C => "c",
            SupportedLanguage::Cpp => "cpp",
            SupportedLanguage::Go => "go",
        }
    }

    /// Get all supported languages
    pub fn all() -> Vec<SupportedLanguage> {
        vec![
            SupportedLanguage::Rust,
            SupportedLanguage::JavaScript,
            SupportedLanguage::TypeScript,
            SupportedLanguage::Tsx,
            SupportedLanguage::Python,
            SupportedLanguage::Java,
            SupportedLanguage::C,
            SupportedLanguage::Cpp,
            SupportedLanguage::Go,
        ]
    }

    /// Get language names as strings
    pub fn all_names() -> Vec<&'static str> {
        Self::all().iter().map(|lang| lang.name()).collect()
    }
}

impl std::fmt::Display for SupportedLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl std::str::FromStr for SupportedLanguage {
    type Err = AnalyzerError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "rust" | "rs" => Ok(SupportedLanguage::Rust),
            "javascript" | "js" => Ok(SupportedLanguage::JavaScript),
            "typescript" | "ts" => Ok(SupportedLanguage::TypeScript),
            "tsx" => Ok(SupportedLanguage::Tsx),
            "python" | "py" => Ok(SupportedLanguage::Python),
            "java" => Ok(SupportedLanguage::Java),
            "c" => Ok(SupportedLanguage::C),
            "cpp" | "c++" | "cxx" => Ok(SupportedLanguage::Cpp),
            "go" | "golang" => Ok(SupportedLanguage::Go),
            _ => Err(AnalyzerError::unsupported_language(s)),
        }
    }
}

/// Trait for mapping language-specific AST node kinds to semantic concepts
pub trait NodeKindMapper {
    /// Check if a node kind represents a function declaration
    fn is_function_node(&self, kind: &str) -> bool;

    /// Check if a node kind represents a class or struct declaration
    fn is_class_node(&self, kind: &str) -> bool;

    /// Check if a node kind represents a control flow statement
    fn is_control_flow_node(&self, kind: &str) -> bool;

    /// Get all function node kinds for this language
    fn function_node_kinds(&self) -> &[&str];

    /// Get all class node kinds for this language
    fn class_node_kinds(&self) -> &[&str];

    /// Get all control flow node kinds for this language (for cyclomatic complexity)
    fn control_flow_node_kinds(&self) -> &[&str];

    /// Check if a node kind represents a comment
    fn is_comment_node(&self, kind: &str) -> bool;

    /// Get all comment node kinds for this language
    fn comment_node_kinds(&self) -> &[&str];

    /// Check if a node kind represents a method (function bound to a class/struct)
    fn is_method_node(&self, kind: &str) -> bool;

    /// Get all method node kinds for this language
    fn method_node_kinds(&self) -> &[&str];
}

impl NodeKindMapper for SupportedLanguage {
    fn is_function_node(&self, kind: &str) -> bool {
        self.function_node_kinds().contains(&kind)
    }

    fn is_class_node(&self, kind: &str) -> bool {
        self.class_node_kinds().contains(&kind)
    }

    fn is_control_flow_node(&self, kind: &str) -> bool {
        self.control_flow_node_kinds().contains(&kind)
    }

    fn function_node_kinds(&self) -> &[&str] {
        match self {
            SupportedLanguage::Rust => &["function_item"],
            SupportedLanguage::JavaScript => &[
                "function_declaration",
                "function_expression",
                "arrow_function",
                "method_definition",
            ],
            SupportedLanguage::TypeScript | SupportedLanguage::Tsx => &[
                "function_declaration",
                "function_expression",
                "arrow_function",
                "method_definition",
                "method_signature",
            ],
            SupportedLanguage::Python => &["function_definition"],
            SupportedLanguage::Java => &["method_declaration", "constructor_declaration"],
            SupportedLanguage::C => &["function_definition"],
            SupportedLanguage::Cpp => &["function_definition", "function_declarator"],
            SupportedLanguage::Go => &["function_declaration", "method_declaration"],
        }
    }

    fn class_node_kinds(&self) -> &[&str] {
        match self {
            SupportedLanguage::Rust => &["struct_item", "enum_item", "impl_item"],
            SupportedLanguage::JavaScript => &["class_declaration"],
            SupportedLanguage::TypeScript | SupportedLanguage::Tsx => {
                &["class_declaration", "interface_declaration"]
            }
            SupportedLanguage::Python => &["class_definition"],
            SupportedLanguage::Java => &[
                "class_declaration",
                "interface_declaration",
                "enum_declaration",
            ],
            SupportedLanguage::C => &["struct_specifier"],
            SupportedLanguage::Cpp => &["class_specifier", "struct_specifier"],
            SupportedLanguage::Go => &["type_declaration"],
        }
    }

    fn control_flow_node_kinds(&self) -> &[&str] {
        match self {
            SupportedLanguage::Rust => &[
                "if_expression",
                "match_expression",
                "for_expression",
                "while_expression",
                "loop_expression",
                "if_let_expression",
                "while_let_expression",
            ],
            SupportedLanguage::JavaScript => &[
                "if_statement",
                "for_statement",
                "for_in_statement",
                "while_statement",
                "do_statement",
                "switch_statement",
                "ternary_expression",
                "try_statement",
                "catch_clause",
            ],
            SupportedLanguage::TypeScript | SupportedLanguage::Tsx => &[
                "if_statement",
                "for_statement",
                "for_in_statement",
                "while_statement",
                "do_statement",
                "switch_statement",
                "ternary_expression",
                "try_statement",
                "catch_clause",
            ],
            SupportedLanguage::Python => &[
                "if_statement",
                "for_statement",
                "while_statement",
                "try_statement",
                "except_clause",
                "with_statement",
                "conditional_expression",
                "list_comprehension",
            ],
            SupportedLanguage::Java => &[
                "if_statement",
                "for_statement",
                "enhanced_for_statement",
                "while_statement",
                "do_statement",
                "switch_expression",
                "try_statement",
                "catch_clause",
                "ternary_expression",
            ],
            SupportedLanguage::C => &[
                "if_statement",
                "for_statement",
                "while_statement",
                "do_statement",
                "switch_statement",
                "conditional_expression",
            ],
            SupportedLanguage::Cpp => &[
                "if_statement",
                "for_statement",
                "for_range_loop",
                "while_statement",
                "do_statement",
                "switch_statement",
                "try_statement",
                "catch_clause",
                "conditional_expression",
            ],
            SupportedLanguage::Go => &[
                "if_statement",
                "for_statement",
                "switch_statement",
                "select_statement",
                "type_switch_statement",
            ],
        }
    }

    fn is_comment_node(&self, kind: &str) -> bool {
        self.comment_node_kinds().contains(&kind)
    }

    fn comment_node_kinds(&self) -> &[&str] {
        match self {
            // Most languages use similar comment node types in tree-sitter
            SupportedLanguage::Rust => &["line_comment", "block_comment"],
            SupportedLanguage::JavaScript => &["comment"],
            SupportedLanguage::TypeScript | SupportedLanguage::Tsx => &["comment"],
            SupportedLanguage::Python => &["comment"],
            SupportedLanguage::Java => &["line_comment", "block_comment"],
            SupportedLanguage::C => &["comment"],
            SupportedLanguage::Cpp => &["comment"],
            SupportedLanguage::Go => &["comment"],
        }
    }

    fn is_method_node(&self, kind: &str) -> bool {
        self.method_node_kinds().contains(&kind)
    }

    fn method_node_kinds(&self) -> &[&str] {
        match self {
            // Rust: functions inside impl blocks are methods
            // Note: tree-sitter marks all as function_item, we rely on parent context
            SupportedLanguage::Rust => &[], // Rust doesn't distinguish at node level
            SupportedLanguage::JavaScript => &["method_definition"],
            SupportedLanguage::TypeScript | SupportedLanguage::Tsx => {
                &["method_definition", "method_signature"]
            }
            SupportedLanguage::Python => &[], // Python uses function_definition for both
            SupportedLanguage::Java => &["method_declaration"],
            SupportedLanguage::C => &[], // C doesn't have methods
            SupportedLanguage::Cpp => &["function_definition"], // Methods are still function_definition
            SupportedLanguage::Go => &["method_declaration"],
        }
    }
}

/// Language detection and parser management
pub struct LanguageManager {
    parsers: HashMap<SupportedLanguage, Parser>,
    enabled_languages: Option<Vec<SupportedLanguage>>,
}

impl Clone for LanguageManager {
    fn clone(&self) -> Self {
        // Create a new LanguageManager with the same enabled languages but fresh parsers
        Self {
            parsers: HashMap::new(),
            enabled_languages: self.enabled_languages.clone(),
        }
    }
}

impl LanguageManager {
    /// Create a new language manager with all languages enabled
    pub fn new() -> Self {
        Self {
            parsers: HashMap::new(),
            enabled_languages: None,
        }
    }

    /// Create a language manager with specific languages enabled
    pub fn with_languages(languages: Vec<SupportedLanguage>) -> Self {
        Self {
            parsers: HashMap::new(),
            enabled_languages: Some(languages),
        }
    }

    /// Get or create a parser for the specified language
    pub fn get_parser(&mut self, lang: SupportedLanguage) -> Result<&mut Parser> {
        // Check if language is enabled
        if let Some(ref enabled) = self.enabled_languages {
            if !enabled.contains(&lang) {
                return Err(AnalyzerError::unsupported_language(lang.name()));
            }
        }

        // Create parser if it doesn't exist
        if let std::collections::hash_map::Entry::Vacant(e) = self.parsers.entry(lang) {
            let mut parser = Parser::new();
            parser.set_language(&lang.get_grammar()).map_err(|e| {
                AnalyzerError::tree_sitter_error(format!(
                    "Failed to load {} grammar: {}",
                    lang.name(),
                    e
                ))
            })?;
            e.insert(parser);
        }

        Ok(self.parsers.get_mut(&lang).unwrap())
    }

    /// Detect language from file path
    pub fn detect_language<P: AsRef<Path>>(&self, path: P) -> Option<SupportedLanguage> {
        let path = path.as_ref();
        let extension = path.extension()?.to_str()?.to_lowercase();

        let language = match extension.as_str() {
            "rs" => SupportedLanguage::Rust,
            "js" | "jsx" | "mjs" | "cjs" => SupportedLanguage::JavaScript,
            "ts" => SupportedLanguage::TypeScript,
            "tsx" => SupportedLanguage::Tsx,
            "py" | "pyw" | "py3" => SupportedLanguage::Python,
            "java" => SupportedLanguage::Java,
            "c" | "h" => SupportedLanguage::C,
            "cpp" | "cc" | "cxx" | "c++" | "hpp" | "hh" | "hxx" => SupportedLanguage::Cpp,
            "go" => SupportedLanguage::Go,
            _ => return None,
        };

        // Check if language is enabled
        if let Some(ref enabled) = self.enabled_languages {
            if enabled.contains(&language) {
                Some(language)
            } else {
                None
            }
        } else {
            Some(language)
        }
    }

    /// Check if a file should be analyzed based on its extension
    pub fn is_supported_file<P: AsRef<Path>>(&self, path: P) -> bool {
        self.detect_language(path).is_some()
    }

    /// Get all enabled languages
    pub fn enabled_languages(&self) -> Vec<SupportedLanguage> {
        self.enabled_languages
            .as_ref()
            .cloned()
            .unwrap_or_else(SupportedLanguage::all)
    }

    /// Get statistics about parser usage
    pub fn parser_stats(&self) -> HashMap<SupportedLanguage, usize> {
        // For now, just return which parsers have been created
        // In a real implementation, you might track usage counts
        self.parsers.keys().map(|&lang| (lang, 1)).collect()
    }
}

impl Default for LanguageManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for language detection
pub fn detect_language_from_path<P: AsRef<Path>>(path: P) -> Option<SupportedLanguage> {
    LanguageManager::new().detect_language(path)
}

/// Get language from string with validation
pub fn language_from_string(s: &str) -> Result<SupportedLanguage> {
    s.parse()
}

/// Validate a list of language strings
pub fn validate_language_list(languages: &[String]) -> Result<Vec<SupportedLanguage>> {
    languages.iter().map(|s| language_from_string(s)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        let manager = LanguageManager::new();

        assert_eq!(
            manager.detect_language("test.rs"),
            Some(SupportedLanguage::Rust)
        );
        assert_eq!(
            manager.detect_language("test.js"),
            Some(SupportedLanguage::JavaScript)
        );
        assert_eq!(
            manager.detect_language("test.py"),
            Some(SupportedLanguage::Python)
        );
        assert_eq!(manager.detect_language("test.txt"), None);
    }

    #[test]
    fn test_tsx_detection() {
        let manager = LanguageManager::new();

        // TSX files should use the Tsx variant (with tsx grammar)
        assert_eq!(
            manager.detect_language("Component.tsx"),
            Some(SupportedLanguage::Tsx)
        );
        // TS files should use TypeScript variant
        assert_eq!(
            manager.detect_language("utils.ts"),
            Some(SupportedLanguage::TypeScript)
        );
        // TSX should report as "typescript" in output
        assert_eq!(SupportedLanguage::Tsx.name(), "typescript");
    }

    #[test]
    fn test_language_from_string() {
        assert!(matches!(
            language_from_string("rust"),
            Ok(SupportedLanguage::Rust)
        ));
        assert!(matches!(
            language_from_string("javascript"),
            Ok(SupportedLanguage::JavaScript)
        ));
        assert!(language_from_string("unknown").is_err());
    }

    #[test]
    fn test_node_kind_mapping() {
        let rust = SupportedLanguage::Rust;
        assert!(rust.is_function_node("function_item"));
        assert!(rust.is_class_node("struct_item"));
        assert!(!rust.is_function_node("struct_item"));

        let js = SupportedLanguage::JavaScript;
        assert!(js.is_function_node("function_declaration"));
        assert!(js.is_class_node("class_declaration"));
    }

    #[test]
    fn test_language_manager_with_restricted_languages() {
        let manager = LanguageManager::with_languages(vec![
            SupportedLanguage::Rust,
            SupportedLanguage::Python,
        ]);

        assert_eq!(
            manager.detect_language("test.rs"),
            Some(SupportedLanguage::Rust)
        );
        assert_eq!(manager.detect_language("test.js"), None);
    }

    #[test]
    fn test_validate_language_list() {
        let languages = vec!["rust".to_string(), "python".to_string()];
        let result = validate_language_list(&languages).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.contains(&SupportedLanguage::Rust));
        assert!(result.contains(&SupportedLanguage::Python));

        let invalid_languages = vec!["rust".to_string(), "unknown".to_string()];
        assert!(validate_language_list(&invalid_languages).is_err());
    }

    #[test]
    fn test_all_languages() {
        let all = SupportedLanguage::all();
        assert!(all.len() >= 8); // We support at least 8 languages
        assert!(all.contains(&SupportedLanguage::Rust));
        assert!(all.contains(&SupportedLanguage::Python));
    }

    #[test]
    fn test_language_display() {
        assert_eq!(SupportedLanguage::Rust.to_string(), "rust");
        assert_eq!(SupportedLanguage::JavaScript.to_string(), "javascript");
    }
}
