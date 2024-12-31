use std::iter::IntoIterator;

use crate::error::{AppError, AppResult};

use super::tokenizer::{tokenize_expr_line, ComparisonType, ExprVariable, Token};


#[derive(Default, Debug, PartialEq)]
/// A single expression for the fuzzer. An example of an expression is `0 <= A <= 1000`.
struct FuzzExpr {
    /// The constant minimum of the expression.
    const_min: i64,
    /// The constant maximum of the expression.
    const_max: i64,
    /// Variable groups declared inside the expression, sorted from right-to-left reversed like how
    /// it's written. For example, `0 <= B <= C,D <= 1000` will give `vec[(C,D), (B)]`.
    vars: Vec<Vec<ExprVariable>>,
    /// Vector of comparisons that we use to modify the maximum constant when picking random
    /// number. Reversed like how it is written. When we encounter a less than comparison, we reduce the maximum random range by 1
    /// (we're talking inclusive range).
    comparisons: Vec<ComparisonType>,
    /// When the expression contains an array, we store it in a separate vector to evaluate later.
    /// This is because the array may contain another variable for the length, and since I don't
    /// want to bother with dependency resolving, this is good enough. However, cases with single
    /// expression like `0 <= A[N]# <= N <= 2000` will still not be allowed (as the `N` is declared
    /// _after_ `A[N]#`).
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
/// Try to parse a vector of tokens from a single line of file into an expression. Consumes the
/// given tokens and moves it into the resulting `FuzzExpr`.
///
/// # Arguments
/// - `tokens`: slice of tokens to parse
///
/// # Returns
/// An `Option` containing a `FuzzExpr` when parsing is successful.
fn parse_expr_from_line(tokens: &mut Vec<Token>) -> Option<FuzzExpr> {
    // Do some sanity checks first: the least amount of valid tokens for a valid expression is 5
    // (e.g `2 <= x <= 10`).
    if tokens.len() < 5 {
        return None
    }
    let mut fuzz_expr = FuzzExpr::default();

    if let Token::NumValue(x) = tokens.pop()? {
        fuzz_expr.const_max = x;
    } else {
        return None
    }

    // Try to parse the first three tokens first.
    if let Token::Comparison(comp) = tokens.pop()? {
        fuzz_expr.comparisons.push(comp);
    } else {
        return None;
    };

    if let Token::VariableGroup(vars) = tokens.pop()? {
        // TODO: maybe this O(n) operation can be improved? This should be fine though as the
        // vector shouldn't contain too many items.

        if !fuzz_expr.contains_array && expr_var_arr_contains_arr_var(&vars) {
            fuzz_expr.contains_array = true;
        }

        fuzz_expr.vars.push(vars);
    } else {
        return None;
    }

    // Parse the rest of the tokens. Parse chunks of two tokens.
    while tokens.len() > 0 {
        if let Token::Comparison(comp) = tokens.pop()? {
            fuzz_expr.comparisons.push(comp);
        } else {
            return None;
        }

        let second_token = tokens.pop()?;
        if let Token::VariableGroup(vars) = second_token {
            if !fuzz_expr.contains_array && expr_var_arr_contains_arr_var(&vars) {
                fuzz_expr.contains_array = true;
            }
            fuzz_expr.vars.push(vars);
        } else if let Token::NumValue(x) = second_token { // last item is a constant so we should stop parsing.
            fuzz_expr.const_min = x;
            return Some(fuzz_expr)
        } else {
            return None
        }
    };

    None

}

/// The whole data used to start the fuzzing. Create one by running `Self::parse`.
#[derive(Debug, PartialEq)]
pub(crate) struct FuzzData {
    /// Vector of valid fuzzer expressions.
    exprs: Vec<FuzzExpr>,
    /// The input order. After all variables have been set in hashmap(s), the strings below will be
    /// used to lookup the variable values from the hashmap.
    input_order: Option<Vec<String>>,
    input_separator: char,
    output_separator: char
}

impl FuzzData {
    /// Parse lines of a file.
    ///
    /// # Arguments
    /// - `input_separator`: the input separator
    /// - `output_separator`: the output separator
    /// - `lines`: an item that can be iterated over as `String`s
    /// 
    /// # Returns
    /// A `AppResult` containing `Self` when parse succeeded. `Err` containing `AppError` otherwise.
    pub(crate) fn parse<T: IntoIterator<Item = String>>(input_separator: char, output_separator: char, lines: T) -> AppResult<Self> {
        let mut exprs = Vec::new();
        let mut input_order = None;
        let mut i = 0;
        for line in lines {
            i += 1;
            if line.starts_with("#") || line.is_empty() {
                continue
            }

            if line.starts_with("input order:") {
                if input_order.is_none() {
                    let tmp_input_order: Vec<&str> = line.split(":").collect();

                if tmp_input_order.len() < 2 {
                    return Err(AppError::InvalidSyntax(i, line))
                }

                let vars: Vec<String> = tmp_input_order[1].split_whitespace().map(|str| str.into()).collect();
                    input_order = Some(vars);
                } else {
                    return Err(AppError::MultipleInputOrder)
                }
                continue;
            }

            // Anything other than the two above are treated as an expression.
            if let Some(mut tokens) = tokenize_expr_line(&line) {
                if let Some(expr) = parse_expr_from_line(&mut tokens) {
                    exprs.push(expr);
                } else {
                    return Err(AppError::InvalidSyntax(i, line))
                };
            } else {
                return Err(AppError::InvalidExpression(i, line))
            }
        }

        if input_order.is_none() {
            return Err(AppError::NoInputOrder);
        }

        Ok(Self {
            input_order,
            exprs,
            input_separator,
            output_separator
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_tokens() {
        // "1 < A[10]# <= C,D <= 100000"
        let mut tokens = vec![Token::NumValue(1),
            Token::Comparison(ComparisonType::LessThan), Token::VariableGroup(vec!["A[10]#".into()]),
            Token::Comparison(ComparisonType::LessThanOrEqualTo),
            Token::VariableGroup(vec!["C".into(), "D".into()]),
            Token::Comparison(ComparisonType::LessThanOrEqualTo), Token::NumValue(100000)];

        let should_be = FuzzExpr {
            contains_array: true,
            vars: vec![vec!["C".into(), "D".into()], vec!["A[10]#".into()]], // reversed
            comparisons: vec![ComparisonType::LessThanOrEqualTo, ComparisonType::LessThanOrEqualTo, ComparisonType::LessThan], // reversed
            const_min: 1,
            const_max: 100000
        };

        let test_parse = parse_expr_from_line(&mut tokens);
        assert_eq!(test_parse, Some(should_be));
    }

    #[test]
    fn test_parse_invalid_tokens() {
        // "< A[10]# <= C,D <= <= 100000"
        let mut tokens = vec![Token::Comparison(ComparisonType::LessThan),
            Token::VariableGroup(vec!["A[10]#".into()]),
            Token::Comparison(ComparisonType::LessThanOrEqualTo),
            Token::VariableGroup(vec!["C".into(), "D".into()]),
            Token::Comparison(ComparisonType::LessThanOrEqualTo),
            Token::Comparison(ComparisonType::LessThanOrEqualTo), Token::NumValue(100000)];

        let test_parse = parse_expr_from_line(&mut tokens);
        assert_eq!(test_parse, None);
    }

    #[test]
    fn test_parse_valid_file() {
        let mut file_string: Vec<String> = Vec::new();
        file_string.push("# Comment".into());
        file_string.push("".into());
        file_string.push("".into());
        file_string.push("1 < A[10]# <= C,D <= 100000".into());
        file_string.push("input order: A C D".into());

        let expr= FuzzExpr {
            contains_array: true,
            vars: vec![vec!["C".into(), "D".into()], vec!["A[10]#".into()]], // reversed
            comparisons: vec![ComparisonType::LessThanOrEqualTo, ComparisonType::LessThanOrEqualTo, ComparisonType::LessThan], // reversed
            const_min: 1,
            const_max: 100000
        };

        let should_be = FuzzData {
            output_separator: '\n',
            input_separator: '\n',
            exprs: vec![expr],
            input_order: Some(vec!["A".into(), "C".into(), "D".into()])
        };

        let result = FuzzData::parse('\n', '\n', file_string.into_iter()).unwrap();

        assert_eq!(result, should_be);
    }

    #[test]
    fn test_parse_invalid_file_syntax() {
        let mut file_string: Vec<String> = Vec::new();
        file_string.push("# Comment".into());
        file_string.push("".into());
        file_string.push("()".into()); // Cannot get tokenized!
        file_string.push("1 < A[10]# <= C,D <= 100000".into());
        file_string.push("input order: A C D".into());

        let result = FuzzData::parse('\n', '\n', file_string.into_iter()).unwrap_err();

        assert_eq!(result, AppError::InvalidExpression(3, "()".into()));
    }

    #[test]
    fn test_parse_invalid_file_expr() {
        let mut file_string: Vec<String> = Vec::new();
        file_string.push("# Comment".into());
        file_string.push("".into());
        file_string.push("< A[10]# <= C,D <= 100000 <".into()); // Can be tokenized but cannot be parsed
        file_string.push("input order: A C D".into());

        let result = FuzzData::parse('\n', '\n', file_string.into_iter()).unwrap_err();

        assert_eq!(result, AppError::InvalidSyntax(3, "< A[10]# <= C,D <= 100000 <".into()));
    }
}
