// Error reporting module for AutoUI Transpiler
//
// Provides fancy error messages with source code highlighting using miette

use miette::{Diagnostic, NamedSource, SourceSpan};
use std::path::PathBuf;
use thiserror::Error;

/// Transpiler error with source location information
#[derive(Debug, Diagnostic, Error)]
pub enum TranspileError {
    /// Error parsing Auto language file
    #[error("Failed to parse {file}")]
    #[diagnostic(
        code(auto_ui::parse_error),
        help("Check the syntax of your Auto language code")
    )]
    ParseError {
        /// File that failed to parse
        file: String,
        /// Source code
        #[source_code]
        source: NamedSource<String>,
        /// Underlying miette error
        #[related]
        related: Vec<miette::Error>,
    },

    /// File I/O error
    #[error("Failed to {action} {file}")]
    #[diagnostic(
        code(auto_ui::io_error)
    )]
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
    #[error("Failed to generate code: {message}")]
    #[diagnostic(
        code(auto_ui::codegen_error)
    )]
    CodeGenError {
        /// Error message
        message: String,
        /// File being processed
        file: String,
    },
}

/// Convert parser error to TranspileError with source code
pub fn into_transpile_error(
    file_path: &PathBuf,
    source: &str,
    parse_error: miette::Error,
) -> TranspileError {
    let file_name = file_path.display().to_string();

    // Try to extract related errors if it's a MultipleErrors
    let related = if let Some(auto_err) = parse_error.downcast_ref::<auto_lang::error::AutoError>() {
        if let auto_lang::error::AutoError::MultipleErrors { errors, .. } = auto_err {
            errors.clone()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    TranspileError::ParseError {
        file: file_name.clone(),
        source: NamedSource::new(file_name, source.to_string()),
        related,
    }
}

/// Convert string error to TranspileError
pub fn string_to_transpile_error(file_path: &PathBuf, message: String) -> TranspileError {
    TranspileError::CodeGenError {
        message,
        file: file_path.display().to_string(),
    }
}
