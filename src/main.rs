use error::{AppResult, AppResultData};

mod input;
mod parser;
mod error;
mod cli;

fn main() -> AppResult<AppResultData> {
    let args = cli::CLIArgs::parse_check()?;
    println!("{:?}", args.executable_a);
    println!("{:?}", args.executable_b);
    Ok(AppResultData {successful_tests: 100, failed_tests: 0, log_file: None})
}
