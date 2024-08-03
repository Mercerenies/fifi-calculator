
use crate::expr::Expr;
use crate::expr::number::{Number, pow_real};
use crate::expr::function::Function;
use crate::expr::function::table::FunctionTable;
use crate::expr::function::builder::{self, FunctionBuilder};
use crate::expr::prisms;
use crate::expr::simplifier::error::SimplifierError;

use num::{BigInt, Zero, One};

pub fn append_statistics_functions(table: &mut FunctionTable) {
  table.insert(arithmetic_mean());
  table.insert(median());
  table.insert(geometric_mean());
  table.insert(arithmetic_geometric_mean());
  table.insert(harmonic_mean());
}

pub fn arithmetic_mean() -> Function {
  FunctionBuilder::new("mean")
    .add_case(
      // Mean of a vector
      builder::arity_one().of_type(prisms::ExprToVector).and_then(|vec, ctx| {
        if vec.is_empty() {
          ctx.errors.push(SimplifierError::custom_error("mean", "Mean of empty vector"));
          Err(vec)
        } else {
          let len = BigInt::from(vec.len());
          let sum = vec.into_iter().reduce(|a, b| Expr::call("+", vec![a, b])).unwrap();
          Ok(Expr::call("/", vec![sum, Expr::from(len)]))
        }
      })
    )
    .build()
}

pub fn median() -> Function {
  FunctionBuilder::new("median")
    .add_case(
      // Median of a vector
      builder::arity_one().of_type(prisms::expr_to_typed_vector(prisms::expr_to_number())).and_then(|mut vec, ctx| {
        if vec.is_empty() {
          ctx.errors.push(SimplifierError::custom_error("median", "Median of empty vector"));
          Err(vec)
        } else {
          vec.sort();
          let len = vec.len();
          if len % 2 == 0 {
            let b = vec.swap_remove(len / 2);
            let a = vec.swap_remove(len / 2 - 1);
            Ok(Expr::from((a + b) / Number::from(2)))
          } else {
            Ok(Expr::from(vec.swap_remove(len / 2)))
          }
        }
      })
    )
    .build()
}

pub fn geometric_mean() -> Function {
  FunctionBuilder::new("gmean")
    .add_case(
      // Geometric Mean of a vector
      builder::arity_one().of_type(prisms::ExprToVector).and_then(|vec, ctx| {
        if vec.is_empty() {
          ctx.errors.push(SimplifierError::custom_error("gmean", "Geometric mean of empty vector"));
          Err(vec)
        } else {
          let len = BigInt::from(vec.len());
          let sum = vec.into_iter().reduce(|a, b| Expr::call("*", vec![a, b])).unwrap();
          Ok(Expr::call("^", vec![sum, Expr::call("/", vec![Expr::from(1), Expr::from(len)])]))
        }
      })
    )
    .build()
}

pub fn arithmetic_geometric_mean() -> Function {
  FunctionBuilder::new("agmean")
    .add_case(
      // Arithmetic-Geometric Mean of a vector
      builder::arity_one().of_type(prisms::expr_to_typed_vector(prisms::expr_to_number())).and_then(|vec, ctx| {
        if vec.is_empty() {
          ctx.errors.push(SimplifierError::custom_error("agmean", "Arithmetic-geometric mean of empty vector"));
          return Err(vec);
        }
        if vec.iter().any(|x| x < &Number::zero()) {
          ctx.errors.push(SimplifierError::custom_error("agmean", "Arithmetic-geometric mean expects nonnegative values"));
          return Err(vec);
        }
        Ok(Expr::from(agmean(vec)))
      })
    )
    .build()
}

fn agmean(values: Vec<Number>) -> Number {
  const EPSILON: f64 = 0.0000001;

  fn amean(a: &Number, b: &Number) -> Number {
    (a + b) / Number::from(2)
  }
  fn gmean(a: &Number, b: &Number) -> Number {
    // Assumes a and b are nonnegative. Panics if not.
    pow_real(a * b, Number::from(0.5)).unwrap_real()
  }

  assert!(!values.is_empty(), "Precondition failed: agmean got empty vec");
  let mut a = values.iter().fold(Number::zero(), |x, y| x + y) / Number::from(BigInt::from(values.len()));
  let mut g = pow_real(values.iter().fold(Number::one(), |x, y| x * y), Number::from(1.0 / values.len() as f64)).unwrap_real();
  for _ in 0..100 { // Limit total iteration count to 100, just in case we converge really slowly.
    if (&a - &g).abs() < Number::from(EPSILON) {
      break;
    }
    (a, g) = (amean(&a, &g), gmean(&a, &g));
  }
  a
}

pub fn harmonic_mean() -> Function {
  FunctionBuilder::new("hmean")
    .add_case(
      // Harmonic Mean of a vector
      builder::arity_one().of_type(prisms::ExprToVector).and_then(|vec, ctx| {
        if vec.is_empty() {
          ctx.errors.push(SimplifierError::custom_error("hmean", "Arithmetic-geometric mean of empty vector"));
          return Err(vec);
        }
        let len = BigInt::from(vec.len());
        let sum = vec.into_iter()
          .map(|a| Expr::call("/", vec![Expr::from(1), a]))
          .reduce(|a, b| Expr::call("+", vec![a, b]))
          .unwrap();
        Ok(Expr::call("/", vec![Expr::from(len), sum]))
      })
    )
    .build()
}
