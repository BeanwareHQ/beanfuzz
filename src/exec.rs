use std::collections::HashMap;

use rand::{distributions::Uniform, prelude::Distribution, rngs::ThreadRng, thread_rng, Rng};

use crate::{error::{AppError, AppResult}, parser::{parser::{FuzzData, FuzzExpr}, tokenizer::{ComparisonType, ExprVariable, LenExpr}}};

/// Variables that have been assigned values go here.
pub struct VarsData {
    /// Hashmap containing variables as its key and value as its, well, values.
    variables: HashMap<String, i64>,
    /// Hashmap containing variables as its key and value (in the form of arrays) as its, well, values.
    arrays: HashMap<String, Vec<i64>>
}

impl VarsData {
    fn set_var(&mut self, key: &str, val: i64) {
        self.variables.insert(key.to_string(), val);
    }

    fn get_var(&self, key: &str) -> Option<&i64> {
        self.variables.get(key)
    }

    fn set_arr(&mut self, key: &str, val: Vec<i64>) {
        self.arrays.insert(key.to_string(), val);
    }

    fn get_arr(&mut self, key: &str) -> Option<&mut Vec<i64>> {
        self.arrays.get_mut(key)
    }

    fn new() -> Self {
        Self {
            variables: HashMap::new(),
            arrays: HashMap::new(),
        }
    }

}

/// Fill an array to a `VarsData` based on given parameters. Accesses to variable values is
/// possible when the array has length of a specific set variable.
///
/// # Arguments
/// - `rng`: thread-local RNG mutable reference
/// - `data`: the data struct that holds variable values
/// - `size`: length of the array
/// - `min`: minimum value of the array's items
/// - `max`: maximum value of the array's items
fn fill_array(rng: &mut ThreadRng, expr: &FuzzExpr, data: &mut VarsData, key: &str, size: &LenExpr, min: i64, max: i64) -> AppResult<i64> {
    let mut new_vec = Vec::new();
    let range = Uniform::from(min..=max);

    let count = match size {
        LenExpr::Variable(key) => *data.get_var(&key).expect("Failed to retrieve value from variable"),
        LenExpr::Constant(val) => *val,
    };

    if count < 1 {
        return Err(AppError::InvalidArraySize(count, expr.to_string()));
    } else {
        let mut max: i64 = 0;
        for _ in 0..=count {
            let new = range.sample(rng);
            max = max.max(new);
            new_vec.push(new);
        }

        data.set_arr(key, new_vec);
        return Ok(max)
    }
}

fn recurse_set_variables(rng: &mut ThreadRng, expr: &FuzzExpr, data: &mut VarsData) -> AppResult<()> {
    let min = if expr.comparisons[0] == ComparisonType::LessThan {
        expr.const_min + 1
    } else {
        expr.const_min
    };
    _recurse_set_variables(rng, expr, data, 0, min)?;
    Ok(())
}

/// Recursively set variable values from the expressions stack.
/// 
/// # Arguments
/// - `rng`: thread-local RNG mutable reference
/// - `expr`: the current expression we're working with
/// - `data`: struct containing variable hashmaps
/// - `depth`: the current depth
/// - `min`: the minimum value from previous variable's value
///
/// # Returns
/// An AppError when an error occurs. Nothing otherwise.
fn _recurse_set_variables(rng: &mut ThreadRng, expr: &FuzzExpr, data: &mut VarsData, depth: usize, min: i64) -> AppResult<()> {
    let vars_len = expr.vars.len();
    let mut run_min = if expr.comparisons[0] == ComparisonType::LessThan {
        expr.const_min + 1
    } else {
        expr.const_min
    };
    if depth < vars_len {
        run_min = min;
    }
    if depth == vars_len {
        return Ok(())
    }
    let max = expr.const_max - (expr.comparisons[depth + 1..].iter().filter(|x| x == &&ComparisonType::LessThan).count() as i64);
    let range = Uniform::from(run_min..=max);

    let mut n_max = 0; // current max value for the entire VariableGroup

    for i in 0..expr.vars[depth].len() {
        if let ExprVariable::Variable(key) = &expr.vars[depth][i] {
            let randomly_picked = range.sample(rng);
            n_max = n_max.max(randomly_picked);
            data.set_var(&key, randomly_picked);
        } else if let ExprVariable::Array(key, len) = &expr.vars[depth][i] {
            let arr_max = fill_array(rng, expr, data, key, len, run_min, max)?;
            n_max = n_max.max(arr_max);
        }
    }

    let next_min = if expr.comparisons[depth + 1] == ComparisonType::LessThan {
        n_max + 1
    } else {
        n_max
    };

    return _recurse_set_variables(rng, expr, data, depth + 1, next_min);
}

#[cfg(test)]
mod tests {
    use crate::parser::{parser::parse_expr_from_line, tokenizer::tokenize_expr_line};

    use super::*;

    #[test]
    fn fill_variables_1() {
        let expr_str = "1 < A < B <= 100";
        let expr = parse_expr_from_line(expr_str, &mut tokenize_expr_line(expr_str).unwrap()).unwrap();
        let mut data = VarsData::new();

        // Amount of possible values for A and B is 99C2 = 4851. The amount of times we need to
        // draw the values to have at least each possibility once is the harmonic sum up to H4851
        // multiplied by 4851. That's 43971.
        for _ in 0..43971 {
            recurse_set_variables(&mut thread_rng(), &expr, &mut data).unwrap();
            assert!(*data.get_var("B").unwrap() <= 100);
            assert!(*data.get_var("B").unwrap() > 2);
            assert!(*data.get_var("A").unwrap() < 100);
            assert!(*data.get_var("A").unwrap() > 1);
        }
    }

    #[test]
    fn fill_variables_2() {
        let expr_str = "1 < A[10]# < 100";
        let expr = parse_expr_from_line(expr_str, &mut tokenize_expr_line(expr_str).unwrap()).unwrap();
        let mut data = VarsData::new();

        // Amount of possible values for A is 98. The amount of times we need to
        // draw the values to have at least each possibility once is the harmonic sum up to H98
        // multiplied by 98. That's 507.
        for _ in 0..507 {
            recurse_set_variables(&mut thread_rng(), &expr, &mut data).unwrap();
            data.get_arr("A").unwrap().iter().for_each(|item| assert!(*item <= 100));
        }
    }
}
