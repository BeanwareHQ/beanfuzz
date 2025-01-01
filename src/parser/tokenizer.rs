//! Components to tokenize lines. Since none of the components really know the context of the
//! tokenization, they only return `Option<T>`s and the caller can return an `AppError` when it
//! encounters an error with the entire context information known.

use std::collections::VecDeque;

// Who knows maybe someday they'll change, right?
const LESS_THAN: &str = "<";
const LESS_THAN_OR_EQUAL_TO: &str = "<=";

pub(crate) type VariableGroup = Vec<ExprVariable>;

#[derive(Debug)]
#[derive(PartialEq)]
/// Comparison type.
pub(crate) enum ComparisonType {
    LessThan,
    LessThanOrEqualTo
}

/// Enum specifically representing the type of expression used for an array variable's length. For
/// example, `N` is treated as a `Variable` and `100` is treated as a `Constant`.
#[derive(PartialEq, Debug)]
pub(crate) enum LenExpr {
    Variable(String),
    Constant(i64)
}

#[derive(PartialEq, Debug)]
/// Representation of a variable used in expressions.
pub(crate) enum ExprVariable {
    /// An array variable. Contains a `String` which represents its string representation and a
    /// `LenExpr` representing the length of the array.
    Array(String, LenExpr),
    /// A variable holding single value.
    Variable(String)
}

#[derive(Debug, PartialEq)]
/// Token for parsing.
pub(crate) enum Token {
    /// A comparison token, equivalent to either `<` or `<=`.
    Comparison(ComparisonType),

    /// A group of variable names.
    VariableGroup(VariableGroup),

    /// A constant 64-bit integer.
    NumValue(i64)
}

// Do not use for the app! Use the non-panicking function `string_to_variable` instead. This is a
// wrapper for the unit testing, for the sake of convenience.
impl From<&str> for ExprVariable {
    fn from(value: &str) -> Self {
        if let Some(tok) = string_to_variable(value) {
            return tok
        }
        panic!("Failed to convert string into a token")
    }
}

/// Try to parse a string into an expression variable.
///
/// # Arguments
/// - `string`: input string
///
/// # Returns
/// An `Option` containing an `ExprVariable` if value is valid as a variable.
fn string_to_variable(string: &str) -> Option<ExprVariable> {
    if string.ends_with("]#") {
        let new_string = string.strip_suffix("]#")?.to_string();
        let split: Vec<&str> = new_string.split("[").collect();
        if split.len() != 2 {
            return None
        }
        let len = split[1];
        let len_expr;
        if let Ok(x) = len.parse::<i64>() {
            len_expr = LenExpr::Constant(x);
        } else {
            len_expr = LenExpr::Variable(split[1].into())
        }
        return Some(ExprVariable::Array(split[0].into(), len_expr))
    } else if !(string.contains("[") || string.contains("]") || string.contains(" ")) {
        return Some(ExprVariable::Variable(string.into()))
    }
    None

}

/// Tokenize a single value.
///
/// # Arguments
/// - `item`: a string of the value.
///
/// # Returns
/// An `Option` containing a `Token` if value is valid as a token.
pub(crate) fn tokenize(item: &str) -> Option<Token> {
    if item == LESS_THAN {
        return Some(Token::Comparison(ComparisonType::LessThan))
    } else if item == LESS_THAN_OR_EQUAL_TO {
        return Some(Token::Comparison(ComparisonType::LessThanOrEqualTo))
    }

    let mut item_iter = item.bytes();
    let first = item_iter.nth(0)?;
    if first.is_ascii_digit() {
        if let Ok(result) = item.parse::<i64>() {
            return Some(Token::NumValue(result))
        } else {
            return None
        }
    }

    if first.is_ascii_alphabetic() {
        if item.contains(',') {
            let mut tokens = Vec::new();
            for item in item.split(',') {
                tokens.push(string_to_variable(item)?)
            }
            return Some(Token::VariableGroup(tokens))
        } else {
            return Some(Token::VariableGroup(vec![string_to_variable(item)?]))
        }
    }
    None
}

/// Tokenize a line of comparison expression, e.g `"3 < A < 100"`. Caller should return an
/// `AppError::InvalidExpression` when this returns `None`.
///
/// # Arguments
/// - `line`: line of expression
///
/// # Returns
/// An `Option` containing vector of `Token`s when parsing is successful.
pub(crate) fn tokenize_expr_line(line: &str) -> Option<VecDeque<Token>> {
    let mut tokens = VecDeque::new();
    let tokens_val = line.split_whitespace();
    for val in tokens_val {
        if let Some(token) = tokenize(val) {
            tokens.push_back(token);
        } else {
            return None
        }
    }
    Some(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_to_variable() {
        assert_eq!(string_to_variable("variable"), Some(ExprVariable::Variable("variable".into())));
        assert_eq!(string_to_variable("some_variable_123"), Some(ExprVariable::Variable("some_variable_123".into())));
        assert_eq!(string_to_variable("array[100]#"), Some(ExprVariable::Array("array".into(), LenExpr::Constant(100))));
        assert_eq!(string_to_variable("array[N]#"), Some(ExprVariable::Array("array".into(), LenExpr::Variable("N".into()))));
        assert_eq!(string_to_variable("this is invalid"), None);
        assert_eq!(string_to_variable("this[is not valid]"), None);
        assert_eq!(string_to_variable("this_is_not_valid[100]"), None);
        assert_eq!(string_to_variable("this_is_not[]valid"), None);
    }

    #[test]
    fn test_tokenize() {
        assert_eq!(tokenize(" "), None);
        assert_eq!(tokenize("<"), Some(Token::Comparison(ComparisonType::LessThan)));
        assert_eq!(tokenize("<="), Some(Token::Comparison(ComparisonType::LessThanOrEqualTo)));
        assert_eq!(tokenize("A,B"), Some(Token::VariableGroup(vec!["A".into(), "B".into()])));
        assert_eq!(tokenize("123"), Some(Token::NumValue(123)));
        assert_eq!(tokenize("1_invalid_var"), None);
        assert_eq!(tokenize("variable"), Some(Token::VariableGroup(vec!["variable".into()])));
    }

    #[test]
    fn test_tokenize_line() {
        let line = "1 < A <= C,D <= 100000";
        let tokens = tokenize_expr_line(line);
        assert_eq!(tokens, Some(VecDeque::from([Token::NumValue(1), Token::Comparison(ComparisonType::LessThan),
            Token::VariableGroup(vec!["A".into()]), Token::Comparison(ComparisonType::LessThanOrEqualTo),
            Token::VariableGroup(vec!["C".into(), "D".into()]),
            Token::Comparison(ComparisonType::LessThanOrEqualTo),
            Token::NumValue(100000)])));

        let line_invalid = "3.4 < 123 != 2_XYZ";
        assert!(tokenize_expr_line(line_invalid).is_none());
    }
}
