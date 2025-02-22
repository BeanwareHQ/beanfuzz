
mod file_handling;
mod parser;
mod error;
mod cli;
mod exec;

use std::{fs::{OpenOptions}, io::Write};

use error::{AppResult, AppResultData};
use exec::Runner;
use file_handling::get_fuzz_data;

fn main() -> AppResult<AppResultData> {
    let args = cli::CLIArgs::checked_parse()?;
    let data = get_fuzz_data(&args.input_sep, &args.output_sep, &args.fuzz_data_filepath)?;

    let mut log_file = if let Some(path) = &args.log_file {
        Some(OpenOptions::new().create(true).write(true).truncate(true).open(path)?)
    } else {
        None
    };

    if let Some(log_file) = &mut log_file {
        log_file.write(&format!("---------\nBeanfuzz ran with parameters: {}\n---------", &args).into_bytes())?;
    }

    let mut runner = Runner::new(data, args.executable_a, args.executable_b);
    let mut fuzz_result = AppResultData::new(args.log_file);

    for i in 0..args.how_many_times {
        let result = runner.run_once();
        match result {
            Ok(result) => match result {
                exec::RunnerResult::Ok => {
                    fuzz_result.successful_tests += 1;
                    println!("Test #{} succeeded", i+1);
                }
                exec::RunnerResult::Fail(out1, out2) => {
                    fuzz_result.failed_tests += 1;
                    if let Some(log_file) = &mut log_file {
                        println!("Test #{} failed! See log file for details.", i+1);
                        log_file.write(b"\n------------------------\n")?;
                        log_file.write(&format!("Test #{} FAILED.\n", i + 1).into_bytes())?;
                        log_file.write(&format!("Hashmap: {:?}\n\n", runner.get_state()).into_bytes())?;
                        log_file.write(&format!("Executable A output:\n~~~~\n{}\n~~~~\n", out1).into_bytes())?;
                        log_file.write(&format!("Executable B output:\n~~~~\n{}\n~~~~\n", out2).into_bytes())?;
                        log_file.write(b"\n------------------------\n")?;
                    } else {
                        println!("Test #{} failed! Enable logging to see output.", i+1);
                    }
                }
            }
            Err(err) => {
                println!("An error occurred with test #{}: {:?}, skipping..", i+1, err);
                fuzz_result.error_tests += 1; 
            }
        }

    }

    if let Some(log_file) = &mut log_file {
        log_file.write(&format!("{}", &fuzz_result).into_bytes())?;
    }

    Ok(fuzz_result)
}
