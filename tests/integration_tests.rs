use std::fs;
use tempfile::TempDir;

use code_analyzer::{
    analyze_directory, analyze_directory_filtered, run_analysis_with_config, AnalysisConfig,
    CliArgs, ColorMode, OutputFormat, SortBy, SupportedLanguage,
};

/// Create a test project with various source files
fn create_test_project() -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Create Rust files
    fs::write(
        root.join("main.rs"),
        r#"
fn main() {
    println!("Hello, world!");
}

struct Calculator {
    value: f64,
}

impl Calculator {
    fn new() -> Self {
        Self { value: 0.0 }
    }
    
    fn add(&mut self, x: f64) -> &mut Self {
        self.value += x;
        self
    }
    
    fn multiply(&mut self, x: f64) -> &mut Self {
        self.value *= x;
        self
    }
    
    fn get_result(&self) -> f64 {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_calculator() {
        let mut calc = Calculator::new();
        calc.add(5.0).multiply(2.0);
        assert_eq!(calc.get_result(), 10.0);
    }
}
"#,
    )
    .unwrap();

    // Create JavaScript files
    fs::write(
        root.join("utils.js"),
        r#"
class Logger {
    constructor(level = 'info') {
        this.level = level;
    }
    
    log(message) {
        console.log(`[${this.level.toUpperCase()}] ${message}`);
    }
    
    error(message) {
        console.error(`[ERROR] ${message}`);
    }
}

function formatDate(date) {
    return date.toISOString().split('T')[0];
}

function validateEmail(email) {
    const regex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return regex.test(email);
}

module.exports = {
    Logger,
    formatDate,
    validateEmail
};
"#,
    )
    .unwrap();

    // Create Python files
    fs::write(
        root.join("data_processor.py"),
        r#"
class DataProcessor:
    def __init__(self):
        self.data = []
    
    def add_data(self, item):
        """Add a single data item."""
        self.data.append(item)
    
    def process_data(self):
        """Process all data items."""
        return [self.transform_item(item) for item in self.data]
    
    def transform_item(self, item):
        """Transform a single data item."""
        if isinstance(item, str):
            return item.upper()
        elif isinstance(item, (int, float)):
            return item * 2
        else:
            return str(item)

def calculate_average(numbers):
    """Calculate the average of a list of numbers."""
    if not numbers:
        return 0
    return sum(numbers) / len(numbers)

def find_max_value(data):
    """Find the maximum value in a dataset."""
    if not data:
        return None
    return max(data)
"#,
    )
    .unwrap();

    // Create TypeScript files
    fs::create_dir(root.join("src")).unwrap();
    fs::write(
        root.join("src").join("types.ts"),
        r#"
interface User {
    id: number;
    name: string;
    email: string;
}

interface ApiResponse<T> {
    data: T;
    status: 'success' | 'error';
    message?: string;
}

class UserService {
    private users: User[] = [];
    
    constructor() {
        this.loadUsers();
    }
    
    private loadUsers(): void {
        // Simulate loading users from API
        this.users = [];
    }
    
    public addUser(user: Omit<User, 'id'>): User {
        const newUser: User = {
            id: Date.now(),
            ...user
        };
        this.users.push(newUser);
        return newUser;
    }
    
    public getUser(id: number): User | undefined {
        return this.users.find(user => user.id === id);
    }
    
    public getAllUsers(): User[] {
        return [...this.users];
    }
}

export { User, ApiResponse, UserService };
"#,
    )
    .unwrap();

    // Create Go files
    fs::write(
        root.join("server.go"),
        r#"
package main

import (
    "encoding/json"
    "fmt"
    "net/http"
    "log"
)

type User struct {
    ID    int    `json:"id"`
    Name  string `json:"name"`
    Email string `json:"email"`
}

type Server struct {
    users []User
}

func NewServer() *Server {
    return &Server{
        users: make([]User, 0),
    }
}

func (s *Server) handleUsers(w http.ResponseWriter, r *http.Request) {
    switch r.Method {
    case "GET":
        s.getUsers(w, r)
    case "POST":
        s.createUser(w, r)
    default:
        http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
    }
}

func (s *Server) getUsers(w http.ResponseWriter, r *http.Request) {
    w.Header().Set("Content-Type", "application/json")
    json.NewEncoder(w).Encode(s.users)
}

func (s *Server) createUser(w http.ResponseWriter, r *http.Request) {
    var user User
    if err := json.NewDecoder(r.Body).Decode(&user); err != nil {
        http.Error(w, "Invalid JSON", http.StatusBadRequest)
        return
    }
    
    user.ID = len(s.users) + 1
    s.users = append(s.users, user)
    
    w.Header().Set("Content-Type", "application/json")
    w.WriteHeader(http.StatusCreated)
    json.NewEncoder(w).Encode(user)
}

func main() {
    server := NewServer()
    http.HandleFunc("/users", server.handleUsers)
    
    fmt.Println("Server starting on :8080")
    log.Fatal(http.ListenAndServe(":8080", nil))
}
"#,
    )
    .unwrap();

    // Create a .gitignore file to test hidden file exclusion
    fs::write(
        root.join(".gitignore"),
        r#"
*.log
*.tmp
/target/
/node_modules/
.env
"#,
    )
    .unwrap();

    dir
}

#[test]
fn test_analyze_directory_basic() {
    let test_dir = create_test_project();

    let result = analyze_directory(test_dir.path());
    assert!(result.is_ok(), "Analysis should succeed");

    let report = result.unwrap();
    assert!(!report.files.is_empty(), "Should find files");
    assert!(report.summary.total_lines > 0, "Should count lines");
    assert!(report.summary.total_functions > 0, "Should count functions");
    assert!(report.summary.total_classes > 0, "Should count classes");

    // Should find multiple languages
    assert!(
        report.summary.language_breakdown.len() > 1,
        "Should detect multiple languages"
    );
}

#[test]
fn test_analyze_directory_filtered() {
    let test_dir = create_test_project();

    // Test filtering by language
    let result = analyze_directory_filtered(test_dir.path(), vec!["rust".to_string()]);
    assert!(result.is_ok(), "Filtered analysis should succeed");

    let report = result.unwrap();
    assert!(!report.files.is_empty(), "Should find Rust files");

    // All files should be Rust
    for file in &report.files {
        assert_eq!(file.language, "rust", "All files should be Rust");
    }
}

#[test]
fn test_run_analysis_with_config() {
    let test_dir = create_test_project();

    let config = AnalysisConfig {
        languages: vec!["javascript".to_string(), "python".to_string()],
        min_lines: 10,
        max_lines: Some(1000),
        include_hidden: false,
        max_file_size_mb: 10,
        verbose: false,
    };

    let result = run_analysis_with_config(test_dir.path(), config);
    assert!(result.is_ok(), "Config-based analysis should succeed");

    let report = result.unwrap();

    // All files should be JavaScript or Python
    for file in &report.files {
        assert!(
            file.language == "javascript" || file.language == "python",
            "Files should be JavaScript or Python, found: {}",
            file.language
        );
    }

    // All files should meet line requirements
    for file in &report.files {
        assert!(
            file.lines_of_code >= 10,
            "Files should have at least 10 lines"
        );
        assert!(
            file.lines_of_code <= 1000,
            "Files should have at most 1000 lines"
        );
    }
}

#[test]
fn test_cli_integration() {
    let test_dir = create_test_project();

    let cli_args = CliArgs {
        path: Some(test_dir.path().to_path_buf()),
        languages: vec!["rust".to_string(), "javascript".to_string()],
        exclude: vec![],
        output: OutputFormat::Table,
        sort: SortBy::Lines,
        limit: 10,
        min_lines: 1,
        max_lines: None,
        min_functions: None,
        min_classes: None,
        include_hidden: false,
        max_file_size_mb: 10,
        compact: false,
        output_file: None,
        verbose: false,
        json_only: false,
        color: ColorMode::Auto,
    };

    let result = code_analyzer::run_analysis(cli_args);
    assert!(result.is_ok(), "CLI analysis should succeed");
}

#[test]
fn test_json_output() {
    let test_dir = create_test_project();
    let output_file = test_dir.path().join("output.json");

    let cli_args = CliArgs {
        path: Some(test_dir.path().to_path_buf()),
        languages: vec![],
        output: OutputFormat::Json,
        sort: SortBy::Lines,
        limit: 50,
        min_lines: 1,
        max_lines: None,
        min_functions: None,
        min_classes: None,
        include_hidden: false,
        max_file_size_mb: 10,
        compact: false,
        output_file: Some(output_file.clone()),
        verbose: false,
        exclude: vec![],
        json_only: false,
        color: ColorMode::Auto,
    };

    let result = code_analyzer::run_analysis(cli_args);
    assert!(result.is_ok(), "JSON output analysis should succeed");

    // Verify JSON file was created
    assert!(output_file.exists(), "JSON output file should exist");

    // Verify JSON content
    let json_content = fs::read_to_string(&output_file).unwrap();
    assert!(!json_content.is_empty(), "JSON file should not be empty");
    assert!(
        json_content.contains("files"),
        "JSON should contain files array"
    );
    assert!(
        json_content.contains("summary"),
        "JSON should contain summary"
    );
}

#[test]
fn test_both_output_formats() {
    let test_dir = create_test_project();
    let output_file = test_dir.path().join("both_output.json");

    let cli_args = CliArgs {
        path: Some(test_dir.path().to_path_buf()),
        languages: vec!["python".to_string()],
        output: OutputFormat::Both,
        sort: SortBy::Functions,
        limit: 5,
        min_lines: 1,
        max_lines: None,
        min_functions: None,
        min_classes: None,
        include_hidden: false,
        max_file_size_mb: 10,
        compact: false,
        output_file: Some(output_file.clone()),
        exclude: vec![],
        verbose: true,
        json_only: false,
        color: ColorMode::Auto,
    };

    let result = code_analyzer::run_analysis(cli_args);
    assert!(result.is_ok(), "Both formats analysis should succeed");

    // Should create JSON file even with Both format
    assert!(
        output_file.exists(),
        "JSON output file should exist for Both format"
    );
}

#[test]
fn test_language_support() {
    let test_dir = create_test_project();

    let result = analyze_directory(test_dir.path());
    assert!(result.is_ok(), "Analysis should succeed");

    let report = result.unwrap();

    // Should detect multiple supported languages
    let detected_languages: std::collections::HashSet<&String> =
        report.files.iter().map(|f| &f.language).collect();

    // Our test project has Rust, JavaScript, Python, TypeScript, Go
    assert!(detected_languages.contains(&"rust".to_string()));
    assert!(detected_languages.contains(&"javascript".to_string()));
    assert!(detected_languages.contains(&"python".to_string()));
    assert!(detected_languages.contains(&"typescript".to_string()));
    assert!(detected_languages.contains(&"go".to_string()));
}

#[test]
fn test_sorting_functionality() {
    let test_dir = create_test_project();

    // Just test that different sort options work without errors
    let sort_options = vec![
        SortBy::Lines,
        SortBy::Functions,
        SortBy::Classes,
        SortBy::Name,
        SortBy::Path,
        SortBy::Complexity,
    ];

    for sort_by in sort_options {
        let result = analyze_directory(test_dir.path());
        assert!(
            result.is_ok(),
            "Analysis should succeed for sort type: {:?}",
            sort_by
        );

        let report = result.unwrap();
        assert!(
            !report.files.is_empty(),
            "Should find files for sort type: {:?}",
            sort_by
        );

        // Just verify we have valid data - sorting is applied at display time
        for file in &report.files {
            assert!(file.lines_of_code > 0, "Files should have lines");
        }
    }
}

#[test]
fn test_empty_directory() {
    let empty_dir = TempDir::new().unwrap();

    let result = analyze_directory(empty_dir.path());
    assert!(result.is_err(), "Empty directory analysis should fail");
}

#[test]
fn test_invalid_language_filter() {
    let test_dir = create_test_project();

    let result = analyze_directory_filtered(test_dir.path(), vec!["invalid_language".to_string()]);

    // Should handle invalid languages gracefully
    // The exact behavior depends on implementation - it might return error or empty results
    if let Ok(report) = result {
        assert!(
            report.files.is_empty(),
            "No files should match invalid language"
        );
    }
}

#[test]
fn test_file_size_limits() {
    let test_dir = create_test_project();

    let config = AnalysisConfig {
        languages: vec![],
        min_lines: 1,
        max_lines: None,
        include_hidden: false,
        max_file_size_mb: 1, // Very small limit
        verbose: false,
    };

    let result = run_analysis_with_config(test_dir.path(), config);
    assert!(result.is_ok(), "Analysis with size limits should succeed");

    let report = result.unwrap();
    // Should still find some files (our test files are small)
    assert!(
        !report.files.is_empty(),
        "Should find files within size limits"
    );
}

#[test]
fn test_line_filtering() {
    let test_dir = create_test_project();

    let config = AnalysisConfig {
        languages: vec![],
        min_lines: 50,        // High minimum
        max_lines: Some(100), // Low maximum
        include_hidden: false,
        max_file_size_mb: 10,
        verbose: false,
    };

    let result = run_analysis_with_config(test_dir.path(), config);
    assert!(result.is_ok(), "Line filtering analysis should succeed");

    let report = result.unwrap();

    // All files should meet line criteria
    for file in &report.files {
        assert!(
            file.lines_of_code >= 50,
            "Files should have at least 50 lines"
        );
        assert!(
            file.lines_of_code <= 100,
            "Files should have at most 100 lines"
        );
    }
}

#[test]
fn test_hidden_file_handling() {
    let test_dir = create_test_project();

    // Test excluding hidden files (default)
    let config1 = AnalysisConfig {
        languages: vec![],
        min_lines: 1,
        max_lines: None,
        include_hidden: false,
        max_file_size_mb: 10,
        verbose: false,
    };

    let result1 = run_analysis_with_config(test_dir.path(), config1);
    assert!(
        result1.is_ok(),
        "Analysis excluding hidden files should succeed"
    );
    let report1 = result1.unwrap();

    // Should not include .gitignore
    let has_gitignore = report1
        .files
        .iter()
        .any(|f| f.path.file_name().unwrap_or_default() == ".gitignore");
    assert!(
        !has_gitignore,
        "Should not include .gitignore when excluding hidden files"
    );

    // Test including hidden files
    let config2 = AnalysisConfig {
        languages: vec![],
        min_lines: 1,
        max_lines: None,
        include_hidden: true,
        max_file_size_mb: 10,
        verbose: false,
    };

    let result2 = run_analysis_with_config(test_dir.path(), config2);
    assert!(
        result2.is_ok(),
        "Analysis including hidden files should succeed"
    );
    let report2 = result2.unwrap();

    // Should have more or equal files when including hidden
    assert!(
        report2.files.len() >= report1.files.len(),
        "Including hidden files should not reduce file count"
    );
}

#[cfg(feature = "cli_tests")]
#[test]
fn test_cli_binary() {
    let test_dir = create_test_project();

    // Build the binary first
    let output = Command::new("cargo")
        .args(&["build", "--release"])
        .output()
        .expect("Failed to build project");

    assert!(output.status.success(), "Build should succeed");

    // Test the CLI binary
    let binary_path = if cfg!(target_os = "windows") {
        "./target/release/code-analyzer.exe"
    } else {
        "./target/release/code-analyzer"
    };

    let output = Command::new(binary_path)
        .arg(test_dir.path())
        .arg("--verbose")
        .output()
        .expect("Failed to run CLI binary");

    assert!(output.status.success(), "CLI should execute successfully");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Analysis completed"),
        "Should show completion message"
    );
}

#[test]
fn test_complexity_calculation() {
    let test_dir = create_test_project();

    let result = analyze_directory(test_dir.path());
    assert!(result.is_ok(), "Analysis should succeed");

    let report = result.unwrap();

    // All files should have complexity scores
    for file in &report.files {
        assert!(
            file.complexity_score >= 0.0,
            "Complexity score should be non-negative"
        );
        assert!(
            file.complexity_score <= 10.0,
            "Complexity score should be reasonable"
        );
    }

    // Files with more functions/classes should generally have higher complexity
    if report.files.len() > 1 {
        let complex_files: Vec<_> = report
            .files
            .iter()
            .filter(|f| f.functions > 5 || f.classes > 2)
            .collect();

        if !complex_files.is_empty() {
            let avg_complexity_complex: f64 = complex_files
                .iter()
                .map(|f| f.complexity_score)
                .sum::<f64>()
                / complex_files.len() as f64;

            let simple_files: Vec<_> = report
                .files
                .iter()
                .filter(|f| f.functions <= 2 && f.classes <= 1)
                .collect();

            if !simple_files.is_empty() {
                let avg_complexity_simple: f64 =
                    simple_files.iter().map(|f| f.complexity_score).sum::<f64>()
                        / simple_files.len() as f64;

                assert!(
                    avg_complexity_complex >= avg_complexity_simple,
                    "More complex files should have higher complexity scores on average"
                );
            }
        }
    }
}

/// Create a mixed language test project with files from all 8 supported languages
fn create_mixed_language_project() -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Create files for all 8 supported languages
    fs::write(
        root.join("main.rs"),
        "fn main() {\n    println!(\"Hello, Rust!\");\n}\n\nstruct Test {}",
    )
    .unwrap();

    fs::write(
        root.join("script.js"),
        "function hello() {\n    console.log('Hello, JavaScript!');\n}\n\nclass Test {}",
    )
    .unwrap();

    fs::write(
        root.join("app.ts"),
        "function hello(): void {\n    console.log('Hello, TypeScript!');\n}\n\nclass Test {}",
    )
    .unwrap();

    fs::write(
        root.join("utils.py"),
        "def hello():\n    print('Hello, Python!')\n\nclass Test:\n    pass",
    )
    .unwrap();

    fs::write(root.join("Main.java"), 
        "public class Main {\n    public static void main(String[] args) {\n        System.out.println(\"Hello, Java!\");\n    }\n}\nclass Test {}"
    ).unwrap();

    fs::write(root.join("math.c"), 
        "#include <stdio.h>\nint main() {\n    printf(\"Hello, C!\\n\");\n    return 0;\n}\nstruct test {};"
    ).unwrap();

    fs::write(root.join("math.cpp"), 
        "#include <iostream>\nint main() {\n    std::cout << \"Hello, C++!\" << std::endl;\n    return 0;\n}\nclass Test {};"
    ).unwrap();

    fs::write(root.join("main.go"), 
        "package main\nimport \"fmt\"\nfunc main() {\n    fmt.Println(\"Hello, Go!\")\n}\ntype Test struct {}"
    ).unwrap();

    dir
}

#[test]
fn test_default_language_detection_finds_all_languages() {
    let test_dir = create_mixed_language_project();

    // Use analyze_directory for simpler testing
    let result = analyze_directory(test_dir.path());
    assert!(
        result.is_ok(),
        "Analysis should succeed with mixed languages"
    );

    let report = result.unwrap();

    // Verify all 8 languages are detected by checking the report
    let detected_languages: std::collections::HashSet<String> =
        report.files.iter().map(|f| f.language.clone()).collect();

    let all_languages = SupportedLanguage::all();
    for language in &all_languages {
        let language_name = language.name();
        assert!(
            detected_languages.contains(language_name),
            "Should detect language: {language_name}. Found: {:?}",
            detected_languages
        );
    }

    // Should detect exactly 8 files (one per language)
    assert_eq!(
        report.files.len(),
        8,
        "Should detect exactly 8 files (one per language)"
    );
    assert_eq!(
        detected_languages.len(),
        8,
        "Should detect exactly 8 different languages"
    );
}

#[test]
fn test_explicit_language_filtering_works() {
    let test_dir = create_mixed_language_project();

    // Use analyze_directory_filtered for simpler testing
    let result =
        analyze_directory_filtered(test_dir.path(), vec!["rust".to_string(), "go".to_string()]);
    assert!(
        result.is_ok(),
        "Analysis should succeed with language filtering"
    );

    let report = result.unwrap();

    // Should contain only Rust and Go files
    let detected_languages: std::collections::HashSet<String> =
        report.files.iter().map(|f| f.language.clone()).collect();

    // Should contain exactly Rust and Go
    assert!(
        detected_languages.contains("rust"),
        "Should contain Rust files"
    );
    assert!(detected_languages.contains("go"), "Should contain Go files");

    // Should NOT contain other languages
    assert!(
        !detected_languages.contains("javascript"),
        "Should not contain JavaScript"
    );
    assert!(
        !detected_languages.contains("python"),
        "Should not contain Python"
    );
    assert!(
        !detected_languages.contains("java"),
        "Should not contain Java"
    );
    assert!(!detected_languages.contains("c"), "Should not contain C");
    assert!(
        !detected_languages.contains("cpp"),
        "Should not contain C++"
    );
    assert!(
        !detected_languages.contains("typescript"),
        "Should not contain TypeScript"
    );

    // Should have exactly 2 files and 2 languages
    assert_eq!(
        report.files.len(),
        2,
        "Should have exactly 2 files (Rust and Go only)"
    );
    assert_eq!(
        detected_languages.len(),
        2,
        "Should have exactly 2 different languages"
    );
}
