// Error reporting module for AutoUI Transpiler
//
// Provides error messages with source code context

use std::path::PathBuf;
use thiserror::Error;

/// Transpiler error with source location information
#[derive(Debug, Error)]
pub enum TranspileError {
    /// Error parsing Auto language file
    #[error("Failed to parse {file}: {message}")]
    ParseError {
        /// File that failed to parse
        file: String,
        /// Error message
        message: String,
        /// Source code (for error reporting)
        source: String,
    },

    /// File I/O error
    #[error("Failed to {action} {file}: {error}")]
    IoError {
        /// Action being performed (read/write)
        action: String,
        /// File path
        file: String,
        /// Underlying error
        #[source]
        error: std::io::Error,
    },

    /// Code generation error
    #[error("Failed to generate code for {file}: {message}")]
    CodeGenError {
        /// File being processed
        file: String,
        /// Error message
        message: String,
    },
}

impl TranspileError {
    /// Get source code if available
    pub fn source_code(&self) -> Option<&str> {
        match self {
            TranspileError::ParseError { source, .. } => Some(source),
            _ => None,
        }
    }

    /// Get file path
    pub fn file_path(&self) -> &str {
        match self {
            TranspileError::ParseError { file, .. } => file,
            TranspileError::IoError { file, .. } => file,
            TranspileError::CodeGenError { file, .. } => file,
        }
    }

    /// Get error message
    pub fn error_message(&self) -> &str {
        match self {
            TranspileError::ParseError { message, .. } => message,
            TranspileError::CodeGenError { message, .. } => message,
            TranspileError::IoError { .. } => "I/O error",
        }
    }
}

/// Convert parser error to TranspileError with source code
pub fn into_transpile_error(
    file_path: &PathBuf,
    source: &str,
    parse_error: String,
) -> TranspileError {
    TranspileError::ParseError {
        file: file_path.display().to_string(),
        message: parse_error,
        source: source.to_string(),
    }
}

/// Convert string error to TranspileError
pub fn string_to_transpile_error(file_path: &PathBuf, message: String) -> TranspileError {
    TranspileError::CodeGenError {
        file: file_path.display().to_string(),
        message,
    }
}
