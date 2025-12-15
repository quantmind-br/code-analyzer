Inferred project context:
The Code Analyzer is a CLI tool written in Rust that performs static code analysis using Tree-sitter AST parsing across 8 programming languages (Rust, JS, TS, Python, Java, C, C++, Go). The core functionality is robust, utilizing parallel processing (rayon) and integrating with gitignore for file discovery. There is an active plan (`history/PLAN.md`, `history/TASKS.md`) focused on refining metrics (Cyclomatic Complexity, AST-based comment counting, function/method differentiation) and improving the terminal output UX (relative paths, better visual reporting). The current focus is on **metrics accuracy** and **user experience** improvements.

Proposed features (prioritized)

- **Name:** Implement Real Cyclomatic Complexity Calculation
- **Problem/Opportunity:** The current complexity metric is a simplistic heuristic (`LOC/100 + 0.5*sqrt(functions) + 0.3*sqrt(classes)`) which is less accurate than industry-standard Cyclomatic Complexity (CC). This is already identified as a high-priority item in the existing plan.
- **Feature description:** Refactor the analysis logic to calculate Cyclomatic Complexity based on counting control flow nodes (decision points) using the Tree-sitter AST, significantly improving metric accuracy.
- **Expected value (user/business):** Provides a more meaningful and standard measure of code complexity, making the refactoring priority score (`complexity_score`) more reliable and valuable to users.
- **Complexity (S/M/L):** M
- **Complexity rationale (brief):** Requires implementing language-specific node kind mappings for control flow statements across all 8 supported languages.
- **Risks and dependencies:** Dependency on the existing 'Cyclomatic Complexity' task from `history/PLAN.md`. Risk of incorrect language mappings for control flow nodes.
- **Evidence (files/folders):** history/PLAN.md, history/TASKS.md (Task 2.1), src/analyzer/language.rs (NodeKindMapper trait methods), src/analyzer/parser.rs (calculate_cyclomatic_complexity).
- **Technical impact (likely modules/files):** src/analyzer/language.rs, src/analyzer/parser.rs, src/analyzer/mod.rs (FileAnalysis struct), src/output/terminal.rs, src/output/json.rs.
- **Implementation plan (steps):** 1. Update `NodeKindMapper::control_flow_node_kinds()` in `src/analyzer/language.rs` for all languages. 2. Implement accurate `calculate_cyclomatic_complexity` in `src/analyzer/parser.rs` using iterative AST traversal. 3. Update `FileAnalysis::calculate_complexity()` to utilize the new CC value. 4. Update output modules.
- **Acceptance criteria:** The `FileAnalysis::cyclomatic_complexity` field is calculated as $1 + \text{number of control flow nodes}$ (e.g., `if`, `for`, `while`, `switch`) using AST, instead of the heuristic.
- **Recommended tests:** Unit tests for `control_flow_node_kinds` per language. Integration tests to verify `cyclomatic_complexity` and `complexity_score` values on mixed language sample files.

- **Name:** AST-Based Comment Line Counting
- **Problem/Opportunity:** The current comment line counting uses a heuristic (`is_comment_line`) based on string prefixes, which is prone to errors, especially with multi-line block comments or code lines starting with a comment character inside a string literal. This is a planned task (Task 2.2).
- **Feature description:** Switch to using the Tree-sitter AST to accurately identify and count lines containing comment nodes, improving metric reliability.
- **Expected value (user/business):** More accurate separation of `lines_of_code` from `comment_lines`, providing a clearer picture of codebase size and density.
- **Complexity (S/M/L):** M
- **Complexity rationale (brief):** Requires implementing node kind mappings for comments and logic to count unique lines spanned by comment nodes (complex for multiline comments).
- **Risks and dependencies:** Dependency on the existing 'Contagem de Comentários via AST' task from `history/PLAN.md`. Requires careful implementation of line range tracking in `src/analyzer/parser.rs`.
- **Evidence (files/folders):** history/PLAN.md (Section 1), history/TASKS.md (Task 2.2), src/analyzer/language.rs (NodeKindMapper trait), src/analyzer/parser.rs (count_lines function, count_comment_lines_ast logic).
- **Technical impact (likely modules/files):** src/analyzer/language.rs, src/analyzer/parser.rs.
- **Implementation plan (steps):** 1. Implement `NodeKindMapper::comment_node_kinds()` for all languages in `src/analyzer/language.rs`. 2. Implement `count_comment_lines_ast` in `src/analyzer/parser.rs` to track unique line numbers spanned by comment nodes. 3. Replace heuristic comment counting in `FileParser::parse_file_with_warnings`.
- **Acceptance criteria:** The `FileAnalysis::comment_lines` metric is calculated using AST traversal, matching external tooling results for complex, multi-line comment scenarios.
- **Recommended tests:** Unit tests using `test_mixed_languages` samples that include block comments and escaped strings to ensure accurate counting.

- **Name:** Distinguish Functions vs. Methods
- **Problem/Opportunity:** The current `functions` metric bundles all callable declarations (free functions, class methods, constructors) together, which dilutes the value for object-oriented languages where method count per class is a key metric. This is a planned task (Task 2.3).
- **Feature description:** Separate the counting into `functions` (free/module-level) and `methods` (class/struct-associated) in `FileAnalysis` and update the `ProjectSummary` accordingly.
- **Expected value (user/business):** Provides a better understanding of object-oriented structure and a more useful metric for refactoring candidates (e.g., classes with too many methods).
- **Complexity (S/M/L):** M
- **Complexity rationale (brief):** Requires implementing specific method node kind mappings for class-based languages (JS, TS, Java, Go, C++) and possibly context-aware analysis for Rust/Python where the distinction is implicit.
- **Risks and dependencies:** Dependency on the existing 'Diferenciação de Funções/Métodos' task from `history/PLAN.md`. Rust's AST makes direct detection difficult without parent-node context, which might require changes beyond simple node kind mapping.
- **Evidence (files/folders):** history/PLAN.md (Section 3), history/TASKS.md (Task 2.3), src/analyzer/language.rs (is_method_node, method_node_kinds).
- **Technical impact (likely modules/files):** src/analyzer/language.rs, src/analyzer/parser.rs (FileAnalysis, count_methods, ProjectSummary), src/output/terminal.rs, src/output/json.rs.
- **Implementation plan (steps):** 1. Finalize `method_node_kinds()` in `src/analyzer/language.rs`. 2. Update `FileAnalysis` and `ProjectSummary` structs with `methods` and `total_methods`. 3. Implement `count_methods` and ensure `count_functions` correctly deducts methods, or rely on distinct node kinds. 4. Update output views.
- **Acceptance criteria:** The `FileAnalysis::methods` count accurately reflects class/struct-associated functions, and `FileAnalysis::functions` reflects free functions where applicable. `ProjectSummary` includes `total_methods` and `avg_methods_per_file`.
- **Recommended tests:** Unit tests for `method_node_kinds` in `src/analyzer/language.rs`. Integration tests using OOP sample files (`test_mixed_languages/app.ts`, `Main.java`, `utils.py`, `main.rs`, `math.cpp`) to verify method counts.

- **Name:** Iterative AST Traversal Refactoring
- **Problem/Opportunity:** The current recursive implementation of AST traversal for metric counting (`count_functions`, `count_classes`) poses a risk of stack overflow on large or deeply nested files, impacting reliability (Robustness, Phase 1 in the plan).
- **Feature description:** Replace recursive traversal functions with an iterative approach using `tree-sitter::TreeCursor` to ensure stack-safe operation for large files.
- **Expected value (user/business):** Increased reliability and robustness when analyzing extremely large or syntactically complex files, preventing crashes due to stack overflow.
- **Complexity (S/M/L):** M
- **Complexity rationale (brief):** The core iterative function (`count_nodes_iterative`) is already defined in `src/analyzer/parser.rs`, but implementing it generically and replacing existing recursive calls requires careful state management.
- **Risks and dependencies:** Dependency on the existing 'Travessia Iterativa do AST' task from `history/PLAN.md`. Risk of subtle logic errors during the switch from recursion to iteration.
- **Evidence (files/folders):** history/PLAN.md (Section 4), history/TASKS.md (Task 1.1), src/analyzer/parser.rs (count_nodes_iterative function already present as implementation of desired goal).
- **Technical impact (likely modules/files):** src/analyzer/parser.rs (count_functions, count_classes, count_control_flow replacement).
- **Implementation plan (steps):** 1. Verify `count_nodes_iterative` implementation in `src/analyzer/parser.rs`. 2. Refactor `count_functions`, `count_classes`, and `count_control_flow` to strictly use the iterative helper. 3. Remove/deprecate any prior recursive logic.
- **Acceptance criteria:** Analysis of large, artificially deep ASTs (if test cases are created) completes successfully without stack overflow. Existing metric counts remain accurate.
- **Recommended tests:** Performance and stability tests with large generated files (if applicable). Unit tests to ensure iterative counts match previous (correct) recursive counts.

- **Name:** Structured Warning Collection and Display
- **Problem/Opportunity:** Currently, non-fatal parsing errors or warnings use `eprintln!` in `AnalyzerEngine::analyze_files_parallel` and `FileParser::parse_file_with_warnings`, making it hard to process warnings programmatically or display them consistently in a report (UX, Robustness, Phase 1 in the plan).
- **Feature description:** Collect all non-fatal parse warnings (`ParseWarning` struct) during parallel analysis and include them in the `AnalysisReport` for structured output and display.
- **Expected value (user/business):** Clear, consolidated, and structured reporting of non-fatal issues (syntax errors, encoding problems) at the end of the report, improving transparency and auditability.
- **Complexity (S/M/L):** S
- **Complexity rationale (brief):** The core structs (`ParseWarning`, `AnalysisReport::warnings`) are already defined. Requires updating the logic in `AnalyzerEngine` and `FileParser` to correctly collect warnings using thread-safe containers (`Arc<Mutex<Vec<ParseWarning>>>`).
- **Risks and dependencies:** Dependency on the existing 'Tratamento de Erros Estruturado' task from `history/PLAN.md`. Need to ensure thread safety of the warning collection mechanism.
- **Evidence (files/folders):** history/PLAN.md (Section 5), history/TASKS.md (Task 1.2), src/error.rs (ParseWarning struct), src/analyzer/mod.rs (analyze_files_parallel uses Arc/Mutex, AnalysisReport struct).
- **Technical impact (likely modules/files):** src/error.rs, src/analyzer/mod.rs, src/analyzer/parser.rs, src/output/terminal.rs.
- **Implementation plan (steps):** 1. Implement warning collection using `Arc<Mutex>` in `AnalyzerEngine::analyze_files_parallel`. 2. Ensure `FileParser::parse_file_with_warnings` returns warnings correctly. 3. Implement `TerminalReporter::display_warnings`.
- **Acceptance criteria:** The final `AnalysisReport` contains a list of all non-fatal warnings with file path, type, and message. Warnings are displayed clearly at the end of the terminal report.
- **Recommended tests:** Unit tests for `ParseWarning` creation. Integration tests using files known to contain syntax errors (if available, [ASSUMPTION]) or mock files to ensure warnings are captured and reported.

- **Name:** Custom Refactoring Candidate Thresholds via CLI
- **Problem/Opportunity:** Refactoring candidate thresholds (e.g., `lines >= 500`, `CC >= 20`) are hardcoded in `src/analyzer/parser.rs`, making them non-configurable for users with different coding standards.
- **Feature description:** Introduce new optional CLI flags to override the default thresholds for flagging files as refactoring candidates (e.g., `--max-cc`, `--max-loc`, `--max-funcs`).
- **Expected value (user/business):** Allows users and CI/CD pipelines to customize the tool to match their organization's specific technical debt tolerance and coding standards.
- **Complexity (S/M/L):** M
- **Complexity rationale (brief):** Requires adding new fields to `CliArgs` and modifying `AnalyzerEngine` or a dedicated `AnalysisConfig` struct to pass these values to `identify_refactoring_candidates`.
- **Risks and dependencies:** Must maintain backward compatibility for users who rely on the default hardcoded values.
- **Evidence (files/folders):** src/analyzer/parser.rs (identify_refactoring_candidates function), src/cli.rs (CliArgs structure).
- **Technical impact (likely modules/files):** src/cli.rs, src/analyzer/parser.rs, src/lib.rs.
- **Implementation plan (steps):** 1. Add optional fields (e.g., `max_lines_candidate: Option<usize>`, `max_cc_candidate: Option<usize>`) to `CliArgs`. 2. Pass these limits through the analysis flow into `identify_refactoring_candidates`. 3. Update `identify_refactoring_candidates` to use the passed limits or fallback to the current hardcoded defaults.
- **Acceptance criteria:** Running the analyzer with `--max-cc 10` correctly flags files with CC >= 10 as refactoring candidates, regardless of the default 20.
- **Recommended tests:** Integration tests exercising the new flags and verifying that file filtering is applied correctly according to the custom thresholds.

- **Name:** Enhanced JSON Output Options
- **Problem/Opportunity:** The JSON output logic in `src/output/json.rs` is primarily focused on exporting the full `AnalysisReport`. Users sometimes only need the project summary or a list of filtered files without configuration metadata for lean integration.
- **Feature description:** Introduce flags/options to allow exporting only the `ProjectSummary` or only the `Vec<FileAnalysis>` list, minimizing JSON payload size for simple dashboard/API consumption.
- **Expected value (user/business):** Enables creation of smaller, specialized JSON reports for dedicated dashboards or REST APIs, improving I/O performance and data parsing simplicity.
- **Complexity (S/M/L):** S
- **Complexity rationale (brief):** The core functions `export_files_only` and `export_summary_only` are already defined and tested in `src/output/json.rs`, requiring only the plumbing through `src/output/mod.rs` and new CLI arguments in `src/cli.rs`.
- **Risks and dependencies:** None, minimal impact on existing code.
- **Evidence (files/folders):** src/output/json.rs (export_files_only, export_summary_only functions), src/cli.rs (CliArgs, OutputFormat enum expansion).
- **Technical impact (likely modules/files):** src/cli.rs, src/output/mod.rs, src/lib.rs.
- **Implementation plan (steps):** 1. Add `OutputFormat::SummaryOnly` and `OutputFormat::FilesOnly` variants to `src/cli.rs`. [ASSUMPTION: Current OutputFormat is Table/Json/Both]. 2. Update `OutputManager::generate_output` in `src/output/mod.rs` to call the appropriate exporter method in `JsonExporter`.
- **Acceptance criteria:** A user can run `code-analyzer --output files-only --output-file files.json` and receive a JSON file containing only the array of `FileAnalysis` objects.
- **Recommended tests:** Integration tests verifying the content structure of the new JSON output formats.

- **Name:** Language-Specific Custom Output Template (Advanced)
- **Problem/Opportunity:** The current terminal output is monolithic. For some languages, developers might want to see unique metrics (e.g., number of traits/interfaces in Rust/TS, number of tests for Go/Python) not captured by the main columns.
- **Feature description:** Allow users to define a custom output template/format string via CLI or a configuration file to display language-specific metrics (which would require parsing and calculation first).
- **Expected value (user/business):** High customization for advanced users, enabling display of metrics that matter most for their specific language or project without cluttering the default view.
- **Complexity (S/M/L):** L
- **Complexity rationale (brief):** Requires a parsing layer for the template string, mapping template variables to `FileAnalysis` fields, and new CLI argument parsing. Requires new `FileAnalysis` fields to hold the additional metrics.
- **Risks and dependencies:** Significant change to the `TerminalReporter` rendering logic. Must handle missing data gracefully.
- **Evidence (files/folders):** src/cli.rs, src/output/terminal.rs, src/analyzer/parser.rs (to eventually hold the extra metrics).
- **Technical impact (likely modules/files):** src/cli.rs, src/output/terminal.rs, src/output/mod.rs.
- **Implementation plan (steps):** 1. [ASSUMPTION] Add optional `custom_format_string: Option<String>` to `CliArgs`. 2. Modify `TerminalReporter::format_analysis_table` to accept and process the custom format string. 3. Implement simple template engine (e.g., `{file}`, `{lines}`, `{cc}`, etc.) within `TerminalReporter`.
- **Acceptance criteria:** User can provide `--format-string "{file} has {lines} lines (CC:{cc})"` and the output table (or simple text output) is rendered using that format.
- **Recommended tests:** Unit tests for the template formatting logic. Integration tests to ensure a basic custom format string is correctly applied.

- **Name:** Git Integration for Analysis Delta
- **Problem/Opportunity:** The tool analyzes the entire codebase every time. In a CI/CD environment, it's more valuable to analyze only files changed since the last successful commit (`git diff --name-only`).
- **Feature description:** Add a CLI flag (e.g., `--only-changed-since <commit-ref>`) that uses a Git command to filter the file list provided to the `FileWalker`, only processing files that have been modified.
- **Expected value (user/business):** Significant performance improvement (speed) in CI/CD pipelines and local development by only processing a subset of files, leading to faster feedback loops.
- **Complexity (S/M/L):** M
- **Complexity rationale (brief):** Requires executing external `git` commands (or using a Rust git library [ASSUMPTION: use external command for simplicity]). Integrating the output into the `FileWalker` discovery process.
- **Risks and dependencies:** Dependency on the `git` executable being available. Adds a non-Rust dependency (git CLI) to the workflow.
- **Evidence (files/folders):** src/cli.rs, src/analyzer/walker.rs (FileWalker::discover_files).
- **Technical impact (likely modules/files):** src/cli.rs, src/analyzer/walker.rs, Cargo.toml (No new dependency needed if using `std::process::Command`).
- **Implementation plan (steps):** 1. Add `only_changed_since: Option<String>` field to `CliArgs`. 2. In `AnalyzerEngine::analyze_project`, if the flag is present, execute `git diff --name-only <commit-ref>`. 3. Filter the file list discovered by `FileWalker` using the output of the git command before passing to `analyze_files_parallel`.
- **Acceptance criteria:** Running the analyzer with `--only-changed-since HEAD~1` analyzes only the files modified in the last commit.
- **Recommended tests:** Integration tests using a temporary git repository setup to verify only changed files are included.

- **Name:** Windows Path Normalization for Output
- **Problem/Opportunity:** The tool is built in Rust and aims for cross-platform compatibility, but path display in the terminal is often better with forward slashes (`/`) even on Windows for consistency and readability, especially in CI/CD.
- **Feature description:** Implement logic in `TerminalReporter::format_file_path` to optionally normalize all backslashes (`\`) in file paths to forward slashes (`/`) for terminal output display.
- **Expected value (user/business):** Improved readability and consistency of reports across platforms, making output easier to consume in cross-platform tools like web dashboards.
- **Complexity (S/M/L):** S
- **Complexity rationale (brief):** Simple string replacement or path to string conversion logic in one place.
- **Risks and dependencies:** Care must be taken not to change the internal `PathBuf` structure, only the display string.
- **Evidence (files/folders):** src/output/terminal.rs (format_file_path function).
- **Technical impact (likely modules/files):** src/output/terminal.rs.
- **Implementation plan (steps):** 1. Add a conditional check in `TerminalReporter::format_file_path` (perhaps controlled by a new `cli::ColorMode` like flag). 2. Call `path_str.replace("\\", "/")` on the final display string before truncation/return.
- **Acceptance criteria:** On a Windows machine, running the tool displays file paths using forward slashes (e.g., `src/main.rs` instead of `src\main.rs`). [ASSUMPTION: The user operates in an environment where this is desired].
- **Recommended tests:** Platform-specific unit tests (if Rust's testing tools permit) to verify Windows path separators are correctly replaced.

Quick wins (up to 3)

- **Quick win 1 (UX):** **Replace deprecated `display_top_files` with `display_refactoring_candidates`**: The plan includes replacing the redundant "Top Files" sections with the new "Refactoring Candidates" display. This is high-impact for UX and the code structure already contains the candidates logic.
- **Quick win 2 (UX/Robustness):** **Set `CliArgs` defaults per `PLAN.md`**: Update `src/cli.rs` to set `limit` default to 10 and `sort` default to `Complexity`. This immediately improves UX to focus on top refactoring candidates.
- **Quick win 3 (Architecture):** **Oversize/Unsupported File Count in Summary**: Update `WalkStats` to track the count of files skipped due to size limits (`files_skipped_size`) and unsupported languages (`files_skipped_language`), and display these in the `ProjectSummary` (WalkStats::summary is already printed in verbose mode). This makes the walker more transparent.

Minimum questions (max 5)

No major questions are necessary as the core, high-priority features are clearly laid out in the existing `history/PLAN.md` and `history/TASKS.md` documents, enabling direct action on metric accuracy and robustness.
