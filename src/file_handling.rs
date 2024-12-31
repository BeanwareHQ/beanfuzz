use std::{fs::File, io::{BufRead, BufReader}, path::Path};

use crate::{error::AppResult, parser::parser::FuzzData};

const DEFAULT_INPUT_SEPARATOR: char = '\n';
const DEFAULT_OUTPUT_SEPARATOR: char = '\n';

struct BufReaderLines {
    buf_reader: BufReader<File>,
}

/// Try to open a file and get the data needed for the fuzzing.
///
/// # Arguments
/// - `input_separator`: input separator for the fuzzing data.
/// - `input_separator`: output separator for the fuzzing data.
///
/// # Returns
/// An `AppResult` containing `FuzzData` when parse is successful, an `AppErr` otherwise.
pub fn get_fuzz_data(input_separator: char, output_separator: char, path: &Path) -> AppResult<FuzzData> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();
    for line in reader.lines() {
        lines.push(line?);
    };

    FuzzData::parse(input_separator, output_separator, lines)
}
