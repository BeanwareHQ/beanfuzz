
mod file_handling;
mod parser;
mod error;
mod cli;

use error::{AppResult, AppResultData};
use file_handling::get_fuzz_data;

fn main() -> AppResult<AppResultData> {
    let args = cli::CLIArgs::parse_check()?;
    let data = get_fuzz_data(args.input_sep, args.output_sep, &args.fuzz_data_filepath)?;
    println!("{:?}", data);
    Ok(AppResultData {successful_tests: 100, failed_tests: 0, log_file: None})
}
