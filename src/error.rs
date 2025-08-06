use std::fmt;
use std::path::PathBuf;

/// Comprehensive error type for the code analyzer application
#[derive(Debug)]
pub enum AnalyzerError {
    /// I/O operations failed (file system access, reading files, etc.)
    Io(std::io::Error),

    /// Parsing errors from tree-sitter or other parsing operations
    Parse(String),

    /// Path validation errors (invalid paths, permissions, etc.)
    InvalidPath(PathBuf),

    /// Unsupported programming language encountered
    UnsupportedLanguage(String),

    /// Configuration file or settings errors
    ConfigError(String),

    /// Tree-sitter specific parsing errors
    TreeSitter(String),

    /// JSON serialization/deserialization errors
    Json(serde_json::Error),

    /// Walk/traversal errors from the ignore crate
    Walk(ignore::Error),

    /// Progress reporting or indicator errors
    Progress(String),

    /// General validation errors
    Validation(String),
}

impl fmt::Display for AnalyzerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AnalyzerError::Io(err) => write!(f, "IO error: {err}"),
            AnalyzerError::Parse(msg) => write!(f, "Parse error: {msg}"),
            AnalyzerError::InvalidPath(path) => write!(f, "Invalid path: {}", path.display()),
            AnalyzerError::UnsupportedLanguage(lang) => {
                write!(f, "Unsupported programming language: {lang}")
            }
            AnalyzerError::ConfigError(msg) => write!(f, "Configuration error: {msg}"),
            AnalyzerError::TreeSitter(msg) => write!(f, "Tree-sitter parsing error: {msg}"),
            AnalyzerError::Json(err) => write!(f, "JSON error: {err}"),
            AnalyzerError::Walk(err) => write!(f, "Directory traversal error: {err}"),
            AnalyzerError::Progress(msg) => write!(f, "Progress reporting error: {msg}"),
            AnalyzerError::Validation(msg) => write!(f, "Validation error: {msg}"),
        }
    }
}

impl std::error::Error for AnalyzerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AnalyzerError::Io(err) => Some(err),
            AnalyzerError::Json(err) => Some(err),
            AnalyzerError::Walk(err) => Some(err),
            _ => None,
        }
    }
}

// Implement From trait for automatic error conversions
impl From<std::io::Error> for AnalyzerError {
    fn from(err: std::io::Error) -> Self {
        AnalyzerError::Io(err)
    }
}

impl From<serde_json::Error> for AnalyzerError {
    fn from(err: serde_json::Error) -> Self {
        AnalyzerError::Json(err)
    }
}

impl From<ignore::Error> for AnalyzerError {
    fn from(err: ignore::Error) -> Self {
        AnalyzerError::Walk(err)
    }
}

/// Convenience Result type alias for the analyzer
pub type Result<T> = std::result::Result<T, AnalyzerError>;

/// Helper functions for creating common errors
impl AnalyzerError {
    /// Create a parse error with context
    pub fn parse_error<S: Into<String>>(msg: S) -> Self {
        AnalyzerError::Parse(msg.into())
    }

    /// Create a tree-sitter error with context
    pub fn tree_sitter_error<S: Into<String>>(msg: S) -> Self {
        AnalyzerError::TreeSitter(msg.into())
    }

    /// Create an invalid path error
    pub fn invalid_path<P: Into<PathBuf>>(path: P) -> Self {
        AnalyzerError::InvalidPath(path.into())
    }

    /// Create an unsupported language error
    pub fn unsupported_language<S: Into<String>>(lang: S) -> Self {
        AnalyzerError::UnsupportedLanguage(lang.into())
    }

    /// Create a configuration error
    pub fn config_error<S: Into<String>>(msg: S) -> Self {
        AnalyzerError::ConfigError(msg.into())
    }

    /// Create a validation error
    pub fn validation_error<S: Into<String>>(msg: S) -> Self {
        AnalyzerError::Validation(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_error_display() {
        let err = AnalyzerError::parse_error("Invalid syntax");
        assert_eq!(err.to_string(), "Parse error: Invalid syntax");

        let err = AnalyzerError::unsupported_language("kotlin");
        assert_eq!(err.to_string(), "Unsupported programming language: kotlin");

        let err = AnalyzerError::invalid_path("/nonexistent/path");
        assert!(err.to_string().contains("Invalid path"));
    }

    #[test]
    fn test_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let analyzer_err: AnalyzerError = io_err.into();

        match analyzer_err {
            AnalyzerError::Io(_) => (), // Expected
            _ => panic!("Expected Io variant"),
        }
    }

    #[test]
    fn test_result_type_alias() {
        fn test_function() -> Result<String> {
            Ok("success".to_string())
        }

        assert!(test_function().is_ok());
    }
}
