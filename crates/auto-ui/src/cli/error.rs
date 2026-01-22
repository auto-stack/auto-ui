// Error reporting module for AutoUI Transpiler
//
// Provides miette-based error reporting with source code context

use std::path::PathBuf;
use miette::{Diagnostic, LabeledSpan, NamedSource, SourceSpan};
use thiserror::Error;

/// Transpiler error with detailed source location information
#[derive(Debug, Error)]
pub enum TranspileError {
    /// Error parsing Auto language file with precise location
    #[error("Failed to parse {file}")]
    ParseError {
        /// File that failed to parse
        file: String,
        /// Source code for error reporting
        source_code: String,
        /// Span of the error in the source code
        span: SourceSpan,
    },

    /// Parse error without exact location (fallback)
    #[error("Failed to parse {file}: {message}")]
    ParseErrorGeneric {
        /// File that failed to parse
        file: String,
        /// Error message
        message: String,
    },

    /// File I/O error
    #[error("Failed to {action} {file}")]
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

impl Diagnostic for TranspileError {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        match self {
            TranspileError::ParseError { .. } | TranspileError::ParseErrorGeneric { .. } => {
                Some(Box::new("auto_ui::parse_error"))
            }
            TranspileError::IoError { .. } => Some(Box::new("auto_ui::io_error")),
            TranspileError::CodeGenError { .. } => Some(Box::new("auto_ui::codegen_error")),
        }
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        match self {
            TranspileError::ParseError { span, .. } => {
                Some(Box::new(std::iter::once(LabeledSpan::new_with_span(
                    Some("error occurred here".to_string()),
                    *span,
                ))))
            }
            _ => None,
        }
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        match self {
            TranspileError::ParseError { .. } => {
                Some(Box::new("Check the syntax around the marked location"))
            }
            TranspileError::ParseErrorGeneric { .. } => Some(Box::new(
                "Make sure all braces are properly closed and keywords are spelled correctly",
            )),
            _ => None,
        }
    }
}

impl TranspileError {
    /// Create a parse error with source code and location
    pub fn parse_error_with_span(
        file_path: &PathBuf,
        source_code: String,
        offset: usize,
        length: usize,
    ) -> Self {
        TranspileError::ParseError {
            file: file_path.display().to_string(),
            source_code,
            span: SourceSpan::new(offset.into(), length),
        }
    }

    /// Create a generic parse error without exact location
    pub fn parse_error_generic(file_path: &PathBuf, message: String) -> Self {
        TranspileError::ParseErrorGeneric {
            file: file_path.display().to_string(),
            message,
        }
    }

    /// Create an I/O error
    pub fn io_error(action: &str, file_path: &PathBuf, error: std::io::Error) -> Self {
        TranspileError::IoError {
            action: action.to_string(),
            file: file_path.display().to_string(),
            error,
        }
    }

    /// Create a code generation error
    pub fn codegen_error(file_path: &PathBuf, message: String) -> Self {
        TranspileError::CodeGenError {
            file: file_path.display().to_string(),
            message,
        }
    }

    /// Get file path
    pub fn file_path(&self) -> &str {
        match self {
            TranspileError::ParseError { file, .. } => file,
            TranspileError::ParseErrorGeneric { file, .. } => file,
            TranspileError::IoError { file, .. } => file,
            TranspileError::CodeGenError { file, .. } => file,
        }
    }

    /// Try to extract error position from error message
    pub fn with_extracted_offset(
        file_path: &PathBuf,
        source_code: String,
        error_msg: &str,
    ) -> Self {
        // Try to extract SourceOffset from error message
        if let Some(start) = error_msg.find("SourceOffset(") {
            let end = error_msg[start..].find(')');
            if let Some(end) = end {
                let num_str = &error_msg[start + 13..start + end];
                if let Ok(offset) = num_str.parse::<usize>() {
                    // Determine length - use 1 for single token errors
                    let length = if error_msg.contains("UnexpectedToken") {
                        1
                    } else {
                        10 // default span length
                    };

                    return TranspileError::parse_error_with_span(
                        file_path,
                        source_code,
                        offset,
                        length,
                    );
                }
            }
        }

        // Fallback to generic error if no offset found
        TranspileError::parse_error_generic(file_path, error_msg.to_string())
    }

    /// Get the source code for miette reporting
    pub fn source_code_for_miette(&self) -> Option<NamedSource<String>> {
        match self {
            TranspileError::ParseError { file, source_code, .. } => {
                Some(NamedSource::new(file.clone(), source_code.clone()))
            }
            _ => None,
        }
    }
}

/// Convert parser error string to TranspileError with source code
pub fn into_transpile_error(
    file_path: &PathBuf,
    source_code: String,
    parse_error: String,
) -> TranspileError {
    TranspileError::with_extracted_offset(file_path, source_code, &parse_error)
}

/// Convert string error to TranspileError
pub fn string_to_transpile_error(file_path: &PathBuf, message: String) -> TranspileError {
    TranspileError::codegen_error(file_path, message)
}
