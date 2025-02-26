use std::{collections::VecDeque, fmt::Display, iter::IntoIterator};

use crate::error::{AppError, AppResult};

use super::tokenizer::{tokenize_expr_line, ComparisonType, ExprVariable, Token, VariableGroup};

#[derive(Default, Debug, PartialEq)]
/// A single expression for the fuzzer. An example of an expression is `0 <= A <= 1000`.
pub(crate) struct FuzzExpr {
    /// The constant minimum of the expression.
    pub(crate) const_min: i64,

    /// The constant maximum of the expression.
    pub(crate) const_max: i64,

    /// Variable groups declared inside the expression. For example, `0 <= B <= C,D <= 1000` will
    /// give `vec[(B), (C, D)]`.
    pub(crate) vars: Vec<VariableGroup>,

    /// Vector of comparisons that we use to modify the maximum constant when picking random
    /// number. When we encounter a less than comparison, we reduce the maximum random range by 1
    /// (we're talking inclusive range).
    pub(crate) comparisons: Vec<ComparisonType>,

    /// When the expression contains an array, we store it in a separate vector to evaluate later.
    /// This is because the array may contain another variable for the length, and since I don't
    /// want to bother with dependency resolving, this is good enough. However, cases with single
    /// expression like `0 <= A[N]# <= N <= 2000` will still not be allowed (as the `N` is declared
    /// _after_ `A[N]#`).
    pub(crate) contains_array: bool,

    /// How many less than's are in the expression. This is used to compute ranges and other stuff.
    pub(crate) less_than_count: u64,

    /// The string representation of the expression. Used for debugging.
    pub(crate) repr: String

}

impl Display for FuzzExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.repr)
    }
    
}

/// Loop through given slice and check if any of its item is an array variable. This is an O(n)
/// operation.
///
/// # Arguments
/// - `slice`: the slice containing `ExprVariable`s.
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

/// Count how many `LessThan` comparisons are found in a slice.
///
/// # Arguments
/// - `slice`: the slice containing `ComparisonType`s.
///
/// # Returns
/// The count of `LessThan` comparison enums.
fn count_less_thans(slice: &[ComparisonType]) -> u64 {
    slice.iter().filter(|x| x == &&ComparisonType::LessThan).count() as u64
}

/// Try to parse a vector of tokens from a single line of file into an expression. Consumes the
/// given tokens (thus the mutable borrow) and moves it into the resulting `FuzzExpr`.
///
/// # Arguments
/// - `tokens`: slice of tokens to parse
///
/// # Returns
/// An `Option` containing a `FuzzExpr` when parsing is successful.
pub(crate) fn parse_expr_from_line(repr: &str, tokens: &mut VecDeque<Token>) -> Option<FuzzExpr> {
    // Do some sanity checks first: the least amount of valid tokens for a valid expression is 5
    // (e.g `2 <= x <= 10`).
    if tokens.len() < 5 {
        return None
    }
    let mut fuzz_expr = FuzzExpr::default();

    fuzz_expr.repr = repr.to_string();

    if let Token::NumValue(x) = tokens.pop_front()? {
        fuzz_expr.const_min = x;
    } else {
        return None
    }

    // Try to parse the first three tokens first.
    if let Token::Comparison(comp) = tokens.pop_front()? {
        fuzz_expr.comparisons.push(comp);
    } else {
        return None;
    };

    if let Token::VariableGroup(vars) = tokens.pop_front()? {
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
        if let Token::Comparison(comp) = tokens.pop_front()? {
            fuzz_expr.comparisons.push(comp);
        } else {
            return None;
        }

        let second_token = tokens.pop_front()?;
        if let Token::VariableGroup(vars) = second_token {
            if !fuzz_expr.contains_array && expr_var_arr_contains_arr_var(&vars) {
                fuzz_expr.contains_array = true;
            }
            fuzz_expr.vars.push(vars);
        } else if let Token::NumValue(x) = second_token { // last item is a constant so we should stop parsing.
            fuzz_expr.const_max = x;

            fuzz_expr.less_than_count = count_less_thans(&fuzz_expr.comparisons);

            // invalid if max is smaller than min
            if x < fuzz_expr.const_min {
                return None
            }

            // also invalid when the possible range cannot fit the variables.
            if (fuzz_expr.const_max - fuzz_expr.const_min).unsigned_abs() < fuzz_expr.less_than_count {
                return None
            }
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
    pub(crate) exprs: Vec<FuzzExpr>,
    /// The input order. After all variables have been set in hashmap(s), the strings below will be
    /// used to lookup the variable values from the hashmap.
    pub(crate) input_order: Vec<String>,
    pub(crate) input_separator: String,
    pub(crate) output_separator: String
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
    /// An `AppResult` containing `Self` when parse succeeded. `Err` containing `AppError` otherwise.
    pub(crate) fn parse<T: IntoIterator<Item = String>>(input_separator: String, output_separator: String, lines: T) -> AppResult<Self> {
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
                if let Some(expr) = parse_expr_from_line(&line, &mut tokens) {
                    exprs.push(expr);
                } else {
                    return Err(AppError::InvalidSyntax(i, line))
                };
            } else {
                return Err(AppError::InvalidExpression(i, line))
            }
        }

        // When an expression contains an array, we have to evaluate them last.
        exprs.sort_by_key(|x| if x.contains_array {1} else {0} );

        Ok(Self {
            input_order: input_order.ok_or(AppError::NoInputOrder)?,
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
        let mut tokens = VecDeque::from([Token::NumValue(1),
            Token::Comparison(ComparisonType::LessThan), Token::VariableGroup(vec!["A[10]#".into()]),
            Token::Comparison(ComparisonType::LessThanOrEqualTo),
            Token::VariableGroup(vec!["C".into(), "D".into()]),
            Token::Comparison(ComparisonType::LessThanOrEqualTo), Token::NumValue(100000)]);

        let should_be = FuzzExpr {
            contains_array: true,
            vars: vec![vec!["A[10]#".into()], vec!["C".into(), "D".into()]],
            comparisons: vec![ComparisonType::LessThan, ComparisonType::LessThanOrEqualTo, ComparisonType::LessThanOrEqualTo],
            const_min: 1,
            const_max: 100000,
            less_than_count: 1,
            repr: "1 < A[10]# <= C,D <= 100000".to_string()
        };

        let test_parse = parse_expr_from_line("1 < A[10]# <= C,D <= 100000", &mut tokens);
        assert_eq!(test_parse, Some(should_be));
    }

    #[test]
    fn test_parse_invalid_tokens() {
        // "< A[10]# <= C,D <= <= 100000"
        let mut tokens = VecDeque::from([Token::Comparison(ComparisonType::LessThan),
            Token::VariableGroup(vec!["A[10]#".into()]),
            Token::Comparison(ComparisonType::LessThanOrEqualTo),
            Token::VariableGroup(vec!["C".into(), "D".into()]),
            Token::Comparison(ComparisonType::LessThanOrEqualTo),
            Token::Comparison(ComparisonType::LessThanOrEqualTo), Token::NumValue(100000)]);

        let test_parse = parse_expr_from_line("< A[10]# <= C,D <= <= 100000", &mut tokens);
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

        let expr = FuzzExpr {
            contains_array: true,
            vars: vec![vec!["A[10]#".into()], vec!["C".into(), "D".into()]],
            comparisons: vec![ComparisonType::LessThan, ComparisonType::LessThanOrEqualTo, ComparisonType::LessThanOrEqualTo],
            const_min: 1,
            const_max: 100000,
            less_than_count: 1,
            repr: "1 < A[10]# <= C,D <= 100000".to_string()
        };

        let should_be = FuzzData {
            output_separator: "\n".to_string(),
            input_separator: "\n".to_string(),
            exprs: vec![expr],
            input_order: vec!["A".into(), "C".into(), "D".into()]
        };

        let result = FuzzData::parse("\n".into(), "\n".into(), file_string.into_iter()).unwrap();

        assert_eq!(result, should_be);
    }

    #[test]
    fn test_parse_invalid_max_bigger_than_min() {
        let mut file_string: Vec<String> = Vec::new();
        file_string.push("# Comment".into());
        file_string.push("".into());
        file_string.push("1000 < A[10]# <= C,D <= 1".into());
        file_string.push("input order: A C D".into());

        let result = FuzzData::parse("\n".into(), "\n".into(), file_string.into_iter()).unwrap_err();

        assert_eq!(result, AppError::InvalidSyntax(3, "1000 < A[10]# <= C,D <= 1".into()));
    }

    #[test]
    fn test_parse_invalid_file_syntax() {
        let mut file_string: Vec<String> = Vec::new();
        file_string.push("# Comment".into());
        file_string.push("".into());
        file_string.push("()".into()); // Cannot get tokenized!
        file_string.push("1 < A[10]# <= C,D <= 100000".into());
        file_string.push("input order: A C D".into());

        let result = FuzzData::parse("\n".into(), "\n".into(), file_string.into_iter()).unwrap_err();

        assert_eq!(result, AppError::InvalidExpression(3, "()".into()));
    }

    #[test]
    fn test_parse_invalid_range() {
        let mut file_string: Vec<String> = Vec::new();
        file_string.push("# Comment".into());
        file_string.push("".into());
        file_string.push("0 < A < B < 2".into());
        file_string.push("input order: A C D".into());

        let result = FuzzData::parse("\n".into(), "\n".into(), file_string.into_iter()).unwrap_err();

        assert_eq!(result, AppError::InvalidSyntax(3, "0 < A < B < 2".into()));
    }

    #[test]
    fn test_parse_invalid_file_expr() {
        let mut file_string: Vec<String> = Vec::new();
        file_string.push("# Comment".into());
        file_string.push("".into());
        file_string.push("< A[10]# <= C,D <= 100000 <".into()); // Can be tokenized but cannot be parsed
        file_string.push("input order: A C D".into());

        let result = FuzzData::parse("\n".into(), "\n".into(), file_string.into_iter()).unwrap_err();

        assert_eq!(result, AppError::InvalidSyntax(3, "< A[10]# <= C,D <= 100000 <".into()));
    }

    #[test]
    fn test_display_repr() {
        let expression = FuzzExpr {
            contains_array: true,
            vars: vec![vec!["A[10]#".into()], vec!["C".into(), "D".into()]],
            comparisons: vec![ComparisonType::LessThan, ComparisonType::LessThanOrEqualTo, ComparisonType::LessThanOrEqualTo],
            const_min: 1,
            const_max: 100000,
            less_than_count: 1,
            repr: "1 < A[10]# <= C,D <= 100000".to_string()
        };

        assert_eq!("1 < A[10]# <= C,D <= 100000", expression.to_string());
    }
}
