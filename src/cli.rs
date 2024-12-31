use std::path::PathBuf;

use clap::Parser;

use crate::error::{AppResult, AppError};

/// Beanfuzz: test output against two executables, used to test competitive programming executables.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct CLIArgs {
    /// Path to the first executable
    #[arg()]
    pub(crate) file_a: PathBuf,

    /// Path to the second executable
    #[arg()]
    pub(crate) file_b: PathBuf,

    /// Input separator
    #[arg(short = 's', default_value = " ")]
    pub(crate) input_sep: char,

    /// Output separator
    #[arg(short = 'o', default_value = " ")]
    pub(crate) ouput_sep: char,
}

impl CLIArgs {
    /// A wrapper function around the `Self::Parse` method. This method returns an
    /// `AppResult<Self>` containing an app error when an argument parsing error occured.
    pub fn parse_check() -> AppResult<Self> {
        let result = Self::parse();
        if !result.file_a.is_file() {
            return Err(AppError::FileNotFound(result.file_a))
        }
        if !result.file_b.is_file() {
            return Err(AppError::FileNotFound(result.file_b))
        }
        Ok(result)
    }
}
