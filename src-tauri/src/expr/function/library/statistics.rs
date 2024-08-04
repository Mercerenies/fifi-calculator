
use crate::expr::Expr;
use crate::expr::number::{Number, ComplexNumber, pow_real};
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
  table.insert(root_mean_square());
  table.insert(sample_std_dev());
  table.insert(pop_std_dev());
  table.insert(sample_variance());
  table.insert(pop_variance());
  table.insert(sample_covariance());
  table.insert(pop_covariance());
  table.insert(correlation());
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

pub fn root_mean_square() -> Function {
  FunctionBuilder::new("rms")
    .add_case(
      // Mean of a vector
      builder::arity_one().of_type(prisms::ExprToVector).and_then(|vec, ctx| {
        if vec.is_empty() {
          ctx.errors.push(SimplifierError::custom_error("rms", "RMS of empty vector"));
          return Err(vec);
        }
        let len = BigInt::from(vec.len());
        let sum = vec.into_iter()
          .map(|x| Expr::call("^", vec![x, Expr::from(2)]))
          .reduce(|a, b| Expr::call("+", vec![a, b]))
          .unwrap();
        Ok(Expr::call("^", vec![
          Expr::call("/", vec![sum, Expr::from(len)]),
          Expr::from(Number::ratio(1, 2)),
        ]))
      })
    )
    .build()
}

pub fn sample_std_dev() -> Function {
  FunctionBuilder::new("stddev")
    .add_case(
      // Sample standard deviation of a vector
      builder::arity_one().of_type(prisms::expr_to_typed_vector(prisms::ExprToComplex)).and_then(|vec, ctx| {
        if vec.len() < 2 {
          ctx.errors.push(SimplifierError::custom_error("stddev", "Sample standard deviation requires at least two elements"));
          return Err(vec);
        }
        let len = vec.len() as i64;
        let vec: Vec<_> = vec.into_iter().map(ComplexNumber::from).collect();
        let mean = vec.iter().fold(ComplexNumber::zero(), |a, b| a + b) / ComplexNumber::from_real(len);
        let sum_of_differences: Number = vec.into_iter()
          .map(|x| (x - &mean).abs_sqr())
          .sum();
        Ok(Expr::from((sum_of_differences / (len - 1)).powf(0.5)))
      })
    )
    .build()
}

pub fn pop_std_dev() -> Function {
  FunctionBuilder::new("pstddev")
    .add_case(
      // Population standard deviation of a vector
      builder::arity_one().of_type(prisms::expr_to_typed_vector(prisms::ExprToComplex)).and_then(|vec, ctx| {
        if vec.is_empty() {
          ctx.errors.push(SimplifierError::custom_error("pstddev", "Population standard deviation on empty vector"));
          return Err(vec);
        }
        let len = vec.len() as i64;
        let vec: Vec<_> = vec.into_iter().map(ComplexNumber::from).collect();
        let mean = vec.iter().fold(ComplexNumber::zero(), |a, b| a + b) / ComplexNumber::from_real(len);
        let sum_of_differences: Number = vec.into_iter()
          .map(|x| (x - &mean).abs_sqr())
          .sum();
        Ok(Expr::from((sum_of_differences / len).powf(0.5)))
      })
    )
    .build()
}

pub fn sample_variance() -> Function {
  FunctionBuilder::new("variance")
    .add_case(
      // Sample variance of a vector
      builder::arity_one().of_type(prisms::expr_to_typed_vector(prisms::ExprToComplex)).and_then(|vec, ctx| {
        if vec.len() < 2 {
          ctx.errors.push(SimplifierError::custom_error("variance", "Variance requires at least two elements"));
          return Err(vec);
        }
        let len = vec.len() as i64;
        let vec: Vec<_> = vec.into_iter().map(ComplexNumber::from).collect();
        let mean = vec.iter().fold(ComplexNumber::zero(), |a, b| a + b) / ComplexNumber::from_real(len);
        let sum_of_differences: Number = vec.into_iter()
          .map(|x| (x - &mean).abs_sqr())
          .sum();
        Ok(Expr::from(sum_of_differences / (len - 1)))
      })
    )
    .build()
}

pub fn pop_variance() -> Function {
  FunctionBuilder::new("pvariance")
    .add_case(
      // Population variance of a vector
      builder::arity_one().of_type(prisms::expr_to_typed_vector(prisms::ExprToComplex)).and_then(|vec, ctx| {
        if vec.is_empty() {
          ctx.errors.push(SimplifierError::custom_error("pvariance", "Population variance of empty vector"));
          return Err(vec);
        }
        let len = vec.len() as i64;
        let vec: Vec<_> = vec.into_iter().map(ComplexNumber::from).collect();
        let mean = vec.iter().fold(ComplexNumber::zero(), |a, b| a + b) / ComplexNumber::from_real(len);
        let sum_of_differences: Number = vec.into_iter()
          .map(|x| (x - &mean).abs_sqr())
          .sum();
        Ok(Expr::from(sum_of_differences / len))
      })
    )
    .build()
}

pub fn sample_covariance() -> Function {
  FunctionBuilder::new("covariance")
    .add_case(
      // Sample covariance of two vectors
      builder::arity_two().both_of_type(prisms::expr_to_typed_vector(prisms::ExprToComplex)).and_then(|x, y, ctx| {
        if x.len() != y.len() {
          ctx.errors.push(SimplifierError::custom_error("covariance", "Covariance requires the same number of elements in both vectors"));
          return Err((x, y));
        }
        if x.len() < 2 {
          ctx.errors.push(SimplifierError::custom_error("covariance", "Covariance requires at least two elements in each vector"));
          return Err((x, y));
        }
        let len = x.len() as i64;
        let x: Vec<_> = x.into_iter().map(ComplexNumber::from).collect();
        let y: Vec<_> = y.into_iter().map(ComplexNumber::from).collect();
        Ok(Expr::from(covar_sum_of_differences(x, y) / (len - 1)))
      })
    )
    .build()
}

pub fn pop_covariance() -> Function {
  FunctionBuilder::new("pcovariance")
    .add_case(
      // Population covariance of two vectors
      builder::arity_two().both_of_type(prisms::expr_to_typed_vector(prisms::ExprToComplex)).and_then(|x, y, ctx| {
        if x.len() != y.len() {
          ctx.errors.push(SimplifierError::custom_error("pcovariance", "Pop covar requires the same number of elements in both vectors"));
          return Err((x, y));
        }
        if x.is_empty() {
          ctx.errors.push(SimplifierError::custom_error("pcovariance", "Pop covar got empty vector"));
          return Err((x, y));
        }
        let len = x.len() as i64;
        let x: Vec<_> = x.into_iter().map(ComplexNumber::from).collect();
        let y: Vec<_> = y.into_iter().map(ComplexNumber::from).collect();
        Ok(Expr::from(covar_sum_of_differences(x, y) / len))
      })
    )
    .build()
}

pub fn correlation() -> Function {
  FunctionBuilder::new("corr")
    .add_case(
      // Correlation of two vectors
      builder::arity_two().both_of_type(prisms::expr_to_typed_vector(prisms::ExprToComplex)).and_then(|x, y, ctx| {
        if x.len() != y.len() {
          ctx.errors.push(SimplifierError::custom_error("corr", "Correlation requires the same number of elements in both vectors"));
          return Err((x, y));
        }
        if x.is_empty() {
          ctx.errors.push(SimplifierError::custom_error("corr", "Correlation got empty vector"));
          return Err((x, y));
        }
        let x: Vec<_> = x.into_iter().map(ComplexNumber::from).collect();
        let y: Vec<_> = y.into_iter().map(ComplexNumber::from).collect();
        let numer = covar_sum_of_differences(x.clone(), y.clone());
        let denom_x = ComplexNumber::from(pow_real(sum_of_sqr_differences(x), Number::from(0.5)));
        let denom_y = ComplexNumber::from(pow_real(sum_of_sqr_differences(y), Number::from(0.5)));
        Ok(Expr::from(numer / (denom_x * denom_y)))
      })
    )
    .build()
}

fn covar_sum_of_differences(x: Vec<ComplexNumber>, y: Vec<ComplexNumber>) -> ComplexNumber {
  assert!(x.len() == y.len(), "Precondition failed: covar_sum_of_differences got vectors of different lengths");
  let len = x.len() as i64;
  let x_mean = x.iter().fold(ComplexNumber::zero(), |a, b| a + b) / ComplexNumber::from_real(len);
  let y_mean = y.iter().fold(ComplexNumber::zero(), |a, b| a + b) / ComplexNumber::from_real(len);
  x.into_iter()
    .zip(y)
    .map(|(x_term, y_term)| (x_term - &x_mean) * (y_term - &y_mean).conj())
    .sum()
}

fn sum_of_sqr_differences(x: Vec<ComplexNumber>) -> Number {
  let len = x.len() as i64;
  let x_mean = x.iter().fold(ComplexNumber::zero(), |a, b| a + b) / ComplexNumber::from_real(len);
  x.into_iter()
    .map(|x_term| (x_term - &x_mean).abs_sqr())
    .sum()
}
