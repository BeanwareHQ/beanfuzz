use std::vec::IntoIter;

use crate::error::{AppError, AppResult};

use super::tokenizer::{ComparisonType, ExprVariable, Token};

const DEFAULT_INPUT_SEPARATOR: char = '\n';
const DEFAULT_OUTPUT_SEPARATOR: char = '\n';

struct FuzzExpr<'a> {
    const_min: u64,
    const_max: u64,
    vars: Vec<&'a ExprVariable>,
    comparisons: Vec<&'a ComparisonType>
}

/// Try to parse a vector of tokens from a single line of file into an expression.
fn parse_expr_from_line(tokens: &[Token]) -> AppResult<FuzzExpr> {
    // The least amount of valid tokens for a valid expression is 5 (e.g `2 <= x <= 10`).
    // The first and last items
    // if tokens.len() < 5 {
    //     return Err(AppError::UnexpectedToken)
    // }
    // if let Token::NumValue(_) = tokens[0];

}

struct FuzzData<'a> {
    exprs: Vec<FuzzExpr<'a>>,
    input_order: Vec<&'a ExprVariable>,
    input_separator: char,
    output_separator: char
}

impl FuzzData<'_> {
    /// Parse lines of a file.
    fn parse(lines: IntoIter<String>) -> AppResult<Self> {
        let mut input_separator = DEFAULT_INPUT_SEPARATOR;
        let mut output_separator = DEFAULT_OUTPUT_SEPARATOR;
        for line in lines {
            if line.starts_with("#") {
                continue
            }
        }

        Ok(Self {
            exprs: todo!(),
            input_order: todo!(),
            input_separator,
            output_separator
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

}
