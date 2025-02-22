use std::{fs::canonicalize, path::PathBuf};

use clap::Parser;

use crate::error::{AppResult, AppError};

/// Beanfuzz: test output against two executables, used to test competitive programming executables.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct CLIArgs {
    /// Path to the fuzzing information
    #[arg()]
    pub(crate) fuzz_data_filepath: PathBuf,

    /// Path to the first executable
    #[arg()]
    pub(crate) executable_a: PathBuf,

    /// Path to the second executable
    #[arg()]
    pub(crate) executable_b: PathBuf,

    /// Input separator
    #[arg(short = 's', default_value = " ")]
    pub(crate) input_sep: String,

    /// Output separator
    #[arg(short = 'o', default_value = " ")]
    pub(crate) output_sep: String,
}

impl CLIArgs {
    /// A wrapper function around the `Self::Parse` method. This method returns an
    /// `AppResult<Self>` containing an app error when an argument parsing error occured.
    pub fn parse_check() -> AppResult<Self> {
        let result = Self::parse();
        if !&result.fuzz_data_filepath.is_file() {
            return Err(AppError::FileNotFound(result.fuzz_data_filepath))
        }

        if !&result.executable_a.is_file() {
            return Err(AppError::FileNotFound(result.executable_a))
        }

        if canonicalize(&result.executable_b)? == canonicalize(&result.executable_a)? {
            return Err(AppError::SameExecutable)
        }

        if !&result.executable_b.is_file() {
            return Err(AppError::FileNotFound(result.executable_b))
        }
        Ok(result)
    }
}
