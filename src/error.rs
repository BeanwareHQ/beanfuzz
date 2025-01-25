use std::fmt::Debug;
use std::process::{ExitCode, Termination};
use std::path::PathBuf;

#[derive(PartialEq)]
pub(crate) enum AppError {
    /// Wrapper for std::io::Error
    IOError(std::io::ErrorKind),

    /// An invalid expression. Contains a `u64` to indicate the line number with the said invalid
    /// expression.
    /// Checked during: tokenization-time
    InvalidExpression(u64, String),

    /// Both executable point to the same path. Contains a `PathBuf` to indicate the same file.
    /// Checked during: CLI args parsing-time
    SameExecutable,


    /// File cannot be found. Contains a `PathBuf` to indicate the nonexistent file.
    /// Checked during: CLI args parsing-time
    FileNotFound(PathBuf),

    /// An invalid syntax (exclusive to fuzz information, i.e the input & output separator and the
    /// input order). Contains a `u64` to indicate the line number along with the said string to
    /// identify the line.
    /// Checked during: parse-time
    InvalidSyntax(u64, String),

    /// When variable is declared twice. Contains `String` indicating the variable name.
    /// Checked during: run-time
    DoubleDeclaration(String),

    /// When variable is not declared but is written in the input order. Contains `String`
    /// indicating the variable name.
    /// Checked during: run-time
    UndeclaredVariable(String),

    /// When there's more than one input order.
    /// Checked during: parse-time
    MultipleInputOrder,

    /// When there's no input order given.
    /// Checked during: parse-time
    NoInputOrder,

    /// When array size is 0 or negative. Contains a `i64` indicating the invalid length and a
    /// `String` indicating the invalid expression
    /// Checked during: execution-time
    InvalidArraySize(i64, String),
}

pub(crate) struct AppResultData {
    /// Amount of tests ran
    pub(crate) successful_tests: u64,

    /// Amount of tests that fails
    pub(crate) failed_tests: u64,

    /// Write test result to log file
    pub(crate) log_file: Option<PathBuf>
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
            Self::InvalidExpression(line, expr) => write!(f, "Invalid expression at line {}: {}", line, expr),
            Self::FileNotFound(file) => write!(f, "File not found: {}", file.display()),
            Self::InvalidSyntax(line, str) => write!(f, "Invalid syntax at line {}: {}", line, str),
            Self::DoubleDeclaration(var) => write!(f, "Variable declared twice: {}", var),
            Self::UndeclaredVariable(var) => write!(f, "Undeclared variable written in input order: {}", var),
            Self::MultipleInputOrder => write!(f, "Input order is declared multiple times"),
            Self::NoInputOrder => write!(f, "No input order given"),
            Self::IOError(kind) => write!(f, "I/O error: {}", kind),
            Self::SameExecutable => write!(f, "Two executables point to the same path"),
            Self::InvalidArraySize(size, expr) => write!(f, "Invalid array size: {} at expression '{}'", size, expr)
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value.kind())
    }
}
