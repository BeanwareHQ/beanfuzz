use std::fmt::Debug;
use std::process::{ExitCode, Termination};
use std::path::PathBuf;

use crate::parser::tokenizer::Token;

pub(crate) enum AppError {
    /// An unexpected token. Contains a `u64` to indicate the line number along with the said
    /// string to identify the token.
    UnexpectedToken(u64, String),

    /// An invalid expression. Contains a `u64` to indicate the line number with the said invalid
    /// expression.
    InvalidExpression(u64, String),

    /// File cannot be found. Contains a `PathBuf` to indicate the nonexistent file.
    FileNotFound(PathBuf)
}

pub(crate) struct AppResultData {
    /// Amount of tests ran
    pub(crate) successful_tests: u64,

    /// Amount of tests that fails
    pub(crate) failed_tests: u64
}

impl Termination for AppResultData {
    fn report(self) -> std::process::ExitCode {
        let mut exit_code = 0;
        if self.failed_tests > 0 {
            println!("--TESTS FINISHED WITH WARNING--\n");
            exit_code = 1;
        } else {
            println!("--TESTS FINISHED--\n");
        }
        println!("Ok: {}", self.successful_tests);
        println!("Failed: {}", self.failed_tests);

        ExitCode::from(exit_code)
    }
}

pub(crate) type AppResult<T> = Result<T, AppError>;

impl Debug for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::UnexpectedToken(line, tok) => write!(f, "Unexpected token at line {}: {}", line, tok),
            AppError::InvalidExpression(line, expr) => write!(f, "Invalid expression at line {}: {}", line, expr),
            AppError::FileNotFound(file) => write!(f, "File not found: {}", file.display()),
        }
    }
}
