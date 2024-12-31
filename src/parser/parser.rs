use std::vec::IntoIter;

use crate::error::{AppError, AppResult};

use super::tokenizer::{ComparisonType, ExprVariable, Token};

const DEFAULT_INPUT_SEPARATOR: char = '\n';
const DEFAULT_OUTPUT_SEPARATOR: char = '\n';

#[derive(Default, Debug, PartialEq)]
/// A single expression for the fuzzer. An example of an expression is `0 <= A <= 1000`.
struct FuzzExpr<'a> {
    /// The constant minimum of the expression.
    const_min: i64,
    /// The constant maximum of the expression.
    const_max: i64,
    /// Variable groups declared inside the expression, sorted from left-to-right exactly like how
    /// it's written. For example, `0 <= B <= C,D <= 1000` will give `vec[(B), (C,D)]`.
    vars: Vec<&'a Vec<ExprVariable>>,
    /// Vector of comparisons that we use to modify the maximum constant when picking random
    /// number. When we encounter a less than comparison, we reduce the maximum random range by 1
    /// (we're talking inclusive range).
    comparisons: Vec<&'a ComparisonType>,
    /// When the expression contains an array, we store it in a separate vector to evaluate later.
    /// This is because the array may contain another variable, and since I don't want to bother
    /// with dependency resolving, this is good enough.
    contains_array: bool
}

/// Loop through given slice and check if any of its item is an array variable. This is an O(n)
/// operation.
///
/// # Arguments
/// - `slice`: The slice containing `ExprVariable`s.
///
/// # Returns
/// A boolean indicating the existence of an array variable inside the slice.
fn expr_var_arr_contains_arr_var(slice: &[ExprVariable]) -> bool {
    for item in slice {
        if let ExprVariable::Array(_, _) = item {
            return true
        }
    }
    false

}
/// Try to parse a vector of tokens from a single line of file into an expression.
///
/// # Arguments
/// - `tokens`: slice of tokens to parse
///
/// # Returns
/// An `Option` containing a `FuzzExpr` when parsing is successful.
fn parse_expr_from_line<'a>(tokens: &'a [Token]) -> Option<FuzzExpr<'a>> {
    // Do some sanity checks first: the least amount of valid tokens for a valid expression is 5
    // (e.g `2 <= x <= 10`).
    if tokens.len() < 5 {
        return None
    }
    let mut fuzz_expr = FuzzExpr::default();

    // Parse from right to left.
    let mut i = tokens.len() - 1 - 3;

    // Try to parse the first three tokens first.
    if let Token::NumValue(x) = tokens.last()? {
        fuzz_expr.const_max = *x;
    } else {
        return None
    }

    if let Token::NumValue(x) = tokens.first()? {
        fuzz_expr.const_min = *x;
    } else {
        return None;
    };

    if let Token::Comparison(comp) = &tokens[i + 2] {
        fuzz_expr.comparisons.push(comp);
    } else {
        return None;
    };

    if let Token::VariableGroup(vars) = &tokens[i + 1] {
        fuzz_expr.vars.push(vars);
        //
        // TODO: maybe this O(n) operation can be improved? This should be fine though as the
        // vector shouldn't contain too many items.

        // Only do the check if the `contains_array` is still `false`.
        if !fuzz_expr.contains_array && expr_var_arr_contains_arr_var(vars) {
            fuzz_expr.contains_array = true;
        }
    } else {
        return None;
    }

    // Parse the rest of the tokens. Parse chunks of two tokens.
    while i >= 1 {
        if let Token::Comparison(comp) = &tokens[i] {
            fuzz_expr.comparisons.push(comp);
        } else {
            return None;
        }

        if let Token::VariableGroup(vars) = &tokens[i - 1] {
            fuzz_expr.vars.push(vars);
            if !fuzz_expr.contains_array && expr_var_arr_contains_arr_var(vars) {
                fuzz_expr.contains_array = true;
            }
        } else if i - 1 == 0 { // last item is a constant so we should stop parsing.
            return Some(fuzz_expr)
        } else {
            return None
        }

        i -= 2;
    };

    None

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

    #[test]
    fn test_parse_tokens() {
        // "1 < A[10] <= C,D <= 100000"
        let tokens = vec![Token::NumValue(1),
            Token::Comparison(ComparisonType::LessThan), Token::VariableGroup(vec!["A[10]#".into()]),
            Token::Comparison(ComparisonType::LessThanOrEqualTo),
            Token::VariableGroup(vec!["C".into(), "D".into()]),
            Token::Comparison(ComparisonType::LessThanOrEqualTo), Token::NumValue(100000)];

        // These cannot exist on their own in the struct as the struct will try to make references
        // to existing variables outside of itself.
        // Outside the test, the `FuzzExpr`'s `vars` vector would have references to the tokens's
        // inner vectors previously made. It has to live as long as the created `Vec<Token>` at the very least.
        let vars_1: Vec<ExprVariable> = vec!["C".into(), "D".into()];
        let vars_2: Vec<ExprVariable> = vec!["A[10]#".into()];

        let should_be = FuzzExpr {
            contains_array: true,
            vars: vec![&vars_1, &vars_2], // reversed
            comparisons: vec![&ComparisonType::LessThanOrEqualTo, &ComparisonType::LessThanOrEqualTo, &ComparisonType::LessThan], // reversed
            const_min: 1,
            const_max: 100000
        };

        let test_parse = parse_expr_from_line(&tokens);
        assert_eq!(test_parse, Some(should_be));
    }
}
