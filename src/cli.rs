use std::{fmt::Display, fs::canonicalize, path::PathBuf};

use clap::Parser;
use is_executable::IsExecutable;

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

    /// Log file
    #[arg(short = 'f', default_value = None)]
    pub(crate) log_file: Option<PathBuf>,

    /// How many times to fuzz
    #[arg(short = 'n', default_value = "100" )]
    pub(crate) how_many_times: u64

}

impl Display for CLIArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = String::new();
        string.push_str(&format!("Fuzzing input path : {:?}\n", self.fuzz_data_filepath));
        string.push_str(&format!("Executable A       : {:?}\n", self.executable_a));
        string.push_str(&format!("Executable B       : {:?}\n", self.executable_b));
        string.push_str(&format!("Input separator    : {:?}\n", self.input_sep));
        string.push_str(&format!("Output separator   : {:?}\n", self.output_sep));
        string.push_str(&format!("Log file path      : {:?}\n", self.log_file));

        write!(f, "{}", string)
    }
}

impl CLIArgs {
    /// A wrapper function around the `Self::Parse` method. This method returns an
    /// `AppResult<Self>` containing an app error when an argument parsing error occured.
    pub fn checked_parse() -> AppResult<Self> {
        let result = Self::parse();
        if !&result.fuzz_data_filepath.is_file() {
            return Err(AppError::FileNotFound(result.fuzz_data_filepath))
        }

        if !&result.executable_a.is_file() {
            return Err(AppError::FileNotFound(result.executable_a))
        }

        if !&result.executable_a.is_executable() {
            return Err(AppError::NotExecutable(result.executable_a))
        }

        if canonicalize(&result.executable_b)? == canonicalize(&result.executable_a)? {
            return Err(AppError::SameExecutable)
        }

        if !&result.executable_b.is_file() {
            return Err(AppError::FileNotFound(result.executable_b))
        }

        if !&result.executable_b.is_executable() {
            return Err(AppError::NotExecutable(result.executable_b))
        }

        Ok(result)
    }
}
