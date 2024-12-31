use error::{AppResult, AppResultData};

mod input;
mod parser;
mod error;
mod cli;

fn main() -> AppResult<AppResultData> {
    let args = cli::CLIArgs::parse_check()?;
    println!("{:?}", args.file_a);
    println!("{:?}", args.file_b);
    Ok(AppResultData {successful_tests: 100, failed_tests: 0})
}
