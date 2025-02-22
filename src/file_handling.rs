use std::{fs::File, io::{BufRead, BufReader}, path::Path};

use crate::{error::AppResult, parser::parser::FuzzData};

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
pub fn get_fuzz_data(input_separator: String, output_separator: String, path: &Path) -> AppResult<FuzzData> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();

    // We can't conveniently turn the Result's into Strings while returning any error encountered,
    // so we loop instead.
    // This should be okay as our fuzz info files aren't supposed to be long.
    // Y'all shouldn't abuse this lil boy—coz we read the whole file to memory.
    for line in reader.lines() {
        lines.push(line?);
    };

    FuzzData::parse(input_separator, output_separator, lines)
}
