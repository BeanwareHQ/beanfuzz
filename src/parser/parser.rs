use std::vec::IntoIter;

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

/// The whole data used to start the fuzzing. Create one by running `Self::new`.
struct FuzzData {
    /// Vector of valid fuzzer expressions.
    exprs: Vec<FuzzExpr>,
    /// The input order. After all variables have been set in hashmap(s), the strings below will be
    /// used to lookup the variable values from the hashmap.
    input_order: Option<Vec<String>>,
    input_separator: char,
    output_separator: char
}

impl FuzzData {
    /// Create a new `FuzzData`.
    ///
    /// # Arguments
    /// - `input_separator`: the input separator.
    /// - `output_separator`: the output separator.
    ///
    /// # Returns
    /// An intiialized `FuzzData`.
    pub fn new(input_separator: char, output_separator: char) -> Self {
        Self {
            exprs: Vec::new(),
            input_order: None,
            input_separator,
            output_separator
        }
    }

    /// Parse lines of a file.
    ///
    /// # Arguments
    /// - `lines`: An iterator over `String`s.
    /// 
    /// # Returns
    /// A `AppResult` containing `Self` when parse succeeded. `Err` containing `AppError` otherwise.
    pub fn parse(&mut self, lines: IntoIter<String>) -> AppResult<Self> {
        let mut i = 1;
        for line in lines {
            if line.starts_with("#") || line.is_empty() {
                continue
            }

            if line.starts_with("input order:") {
                if self.input_order.is_none() {

                let input_order: Vec<&str> = line.split(":").collect();

                if input_order.len() < 2 {
                    return Err(AppError::InvalidSyntax(i, line))
                }

                let vars: Vec<String> = input_order[1].split_whitespace().map(|str| str.into()).collect();
                    self.input_order = Some(vars);
                } else {
                    return Err(AppError::MultipleInputOrder)
                }
                continue;
            }

            // Anything other than the two above are treated as an expression.
            if let Some(mut tokens) = tokenize_expr_line(&line) {
                if let Some(expr) = parse_expr_from_line(&mut tokens) {
                    self.exprs.push(expr);
                } else {
                    return Err(AppError::InvalidSyntax(i, line))
                };
            } else {
                return Err(AppError::InvalidExpression(i, line))
            }
            i += 1;

        }

        Ok(Self {
            exprs: todo!(),
            input_order: todo!(),
            input_separator: self.input_separator,
            output_separator: self.output_separator
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tokens() {
        // "1 < A[10] <= C,D <= 100000"
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
}
