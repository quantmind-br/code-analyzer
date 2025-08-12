use std::fs;
use tempfile::TempDir;
use code_analyzer::{run_analysis, CliArgs, SupportedLanguage};

/// Create a test project with files from all supported languages
fn create_mixed_language_project() -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Create files for all 8 supported languages
    fs::write(root.join("main.rs"), 
        "fn main() {\n    println!(\"Hello, Rust!\");\n}\n\nstruct Test {}"
    ).unwrap();
    
    fs::write(root.join("script.js"), 
        "function hello() {\n    console.log('Hello, JavaScript!');\n}\n\nclass Test {}"
    ).unwrap();
    
    fs::write(root.join("app.ts"), 
        "function hello(): void {\n    console.log('Hello, TypeScript!');\n}\n\nclass Test {}"
    ).unwrap();
    
    fs::write(root.join("utils.py"), 
        "def hello():\n    print('Hello, Python!')\n\nclass Test:\n    pass"
    ).unwrap();
    
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
    
    let cli_args = CliArgs {
        path: Some(test_dir.path().to_path_buf()),
        languages: vec![], // Empty = default behavior
        verbose: false,
        exclude: vec![], // No exclusions
        ..Default::default()
    };

    let result = run_analysis(cli_args);
    assert!(result.is_ok(), "Analysis should succeed with mixed languages");
    
    // Check that output JSON contains all 8 languages
    let json_path = test_dir.path().join("refactor-candidates.json");
    assert!(json_path.exists(), "JSON output should be created");
    
    let json_content = fs::read_to_string(&json_path).unwrap();
    
    // Verify all 8 languages are detected
    let all_languages = SupportedLanguage::all();
    for language in &all_languages {
        let language_name = language.name();
        assert!(json_content.contains(&format!("\"language\":\"{language_name}\"")), 
            "JSON should contain language: {language_name}");
    }
}

#[test]
fn test_explicit_language_filtering_works() {
    let test_dir = create_mixed_language_project();
    
    let cli_args = CliArgs {
        path: Some(test_dir.path().to_path_buf()),
        languages: vec!["rust".to_string(), "go".to_string()], // Only Rust and Go
        verbose: false,
        exclude: vec![], // No exclusions
        ..Default::default()
    };

    let result = run_analysis(cli_args);
    assert!(result.is_ok(), "Analysis should succeed with language filtering");
    
    // Check that output JSON contains only Rust and Go
    let json_path = test_dir.path().join("refactor-candidates.json");
    let json_content = fs::read_to_string(&json_path).unwrap();
    
    // Should contain Rust and Go
    assert!(json_content.contains("\"language\":\"rust\""), "Should contain Rust files");
    assert!(json_content.contains("\"language\":\"go\""), "Should contain Go files");
    
    // Should NOT contain other languages
    assert!(!json_content.contains("\"language\":\"javascript\""), "Should not contain JavaScript");
    assert!(!json_content.contains("\"language\":\"python\""), "Should not contain Python");
    assert!(!json_content.contains("\"language\":\"java\""), "Should not contain Java");
    assert!(!json_content.contains("\"language\":\"c\""), "Should not contain C");
    assert!(!json_content.contains("\"language\":\"cpp\""), "Should not contain C++");
    assert!(!json_content.contains("\"language\":\"typescript\""), "Should not contain TypeScript");
}

#[test]
fn test_single_language_filtering() {
    let test_dir = create_mixed_language_project();
    
    let cli_args = CliArgs {
        path: Some(test_dir.path().to_path_buf()),
        languages: vec!["python".to_string()], // Only Python
        verbose: false,
        exclude: vec![], // No exclusions
        ..Default::default()
    };

    let result = run_analysis(cli_args);
    assert!(result.is_ok(), "Analysis should succeed with single language filtering");
    
    let json_path = test_dir.path().join("refactor-candidates.json");
    let json_content = fs::read_to_string(&json_path).unwrap();
    
    // Should contain only Python
    assert!(json_content.contains("\"language\":\"python\""), "Should contain Python files");
    
    // Count occurrences of "language": to ensure only one type
    let language_count = json_content.matches("\"language\":").count();
    assert_eq!(language_count, 1, "Should have exactly 1 file (Python only)");
}

#[test] 
fn test_invalid_language_filtering() {
    let test_dir = create_mixed_language_project();
    
    let cli_args = CliArgs {
        path: Some(test_dir.path().to_path_buf()),
        languages: vec!["rust".to_string(), "invalid_language".to_string()],
        verbose: false,
        exclude: vec![], // No exclusions
        ..Default::default()
    };

    let result = run_analysis(cli_args);
    assert!(result.is_err(), "Analysis should fail with invalid language");
}

#[test]
fn test_empty_directory_behavior() {
    let test_dir = TempDir::new().unwrap();
    // Empty directory - no files
    
    let cli_args = CliArgs {
        path: Some(test_dir.path().to_path_buf()),
        languages: vec![], // Default behavior
        verbose: false,
        exclude: vec![], // No exclusions
        ..Default::default()
    };

    let result = run_analysis(cli_args);
    assert!(result.is_err(), "Analysis should fail with empty directory");
}

#[test]
fn test_no_supported_files() {
    let test_dir = TempDir::new().unwrap();
    let root = test_dir.path();
    
    // Create unsupported file types
    fs::write(root.join("README.md"), "# Test Project").unwrap();
    fs::write(root.join("config.txt"), "some config").unwrap();
    fs::write(root.join("data.json"), "{\"test\": true}").unwrap();
    
    let cli_args = CliArgs {
        path: Some(test_dir.path().to_path_buf()),
        languages: vec![], // Default behavior
        verbose: false,
        exclude: vec![], // No exclusions
        ..Default::default()
    };

    let result = run_analysis(cli_args);
    assert!(result.is_err(), "Analysis should fail when no supported files found");
}