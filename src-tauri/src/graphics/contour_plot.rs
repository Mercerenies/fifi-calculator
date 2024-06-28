
//! Functionality for producing two-dimensional contour plots of data.

use crate::util::matrix::{Matrix, MatrixDimsError};
use crate::expr::number::{Number, ComplexNumber};
use crate::expr::algebra::{ExprFunction, ExprFunction2};
use crate::expr::prisms::ExprToNumber;
use super::dataset::{XDataSet, LengthError, GenReason};
use super::floatify;

use thiserror::Error;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContourPlotDirective {
  x_values: Vec<f64>,
  y_values: Vec<f64>,
  // Must be a YxX matrix.
  z_values: Matrix<f64>,
}

#[derive(Debug, Error)]
pub enum ContourPlotError {
  #[error("{0}")]
  LengthError(#[from] LengthError),
  #[error("{0}")]
  MatrixDimsError(#[from] MatrixDimsError),
}

impl ContourPlotDirective {
  pub fn empty() -> ContourPlotDirective {
    ContourPlotDirective {
      x_values: vec![],
      y_values: vec![],
      z_values: Matrix::empty(),
    }
  }

  pub fn from_points(
    x_dataset: &XDataSet,
    y_dataset: &XDataSet,
    z_values: Vec<Vec<Number>>,
  ) -> Result<ContourPlotDirective, ContourPlotError> {
    let z_values = Matrix::new(z_values)?;

    let x_values = x_dataset.gen_exact_points(Some(z_values.width()), GenReason::TwoDimensional)?;
    let y_values = y_dataset.gen_exact_points(Some(z_values.height()), GenReason::TwoDimensional)?;

    let x_values: Vec<_> = floatify(x_values);
    let y_values: Vec<_> = floatify(y_values);
    let z_values = z_values.map(|n| n.to_f64_or_nan());

    Ok(Self { x_values, y_values, z_values })
  }

  pub fn from_expr_function2(
    x_dataset: &XDataSet,
    y_dataset: &XDataSet,
    z_function: &ExprFunction2,
  ) -> ContourPlotDirective {
    let x_values = x_dataset.gen_points(GenReason::TwoDimensional);
    let y_values = y_dataset.gen_points(GenReason::TwoDimensional);

    let z_values = Matrix::from_generator(y_values.len(), x_values.len(), |idx| {
      // TODO: If enough of these fail (percentage-wise) we should
      // probably report some sort of generic error to the user.
      match z_function.eval_at_real(x_values[idx.x].clone(), y_values[idx.y].clone()) {
        Err(_) => f64::NAN,
        Ok(res) => res.to_f64_or_nan(),
      }
    });

    let x_values: Vec<_> = floatify(x_values);
    let y_values: Vec<_> = floatify(y_values);

    Self { x_values, y_values, z_values }
  }

  pub fn from_complex_function(
    x_dataset: &XDataSet,
    y_dataset: &XDataSet,
    z_function: &ExprFunction,
  ) -> ContourPlotDirective {
    let x_values = x_dataset.gen_points(GenReason::TwoDimensional);
    let y_values = y_dataset.gen_points(GenReason::TwoDimensional);

    let z_values = Matrix::from_generator(y_values.len(), x_values.len(), |idx| {
      // TODO: If enough of these fail (percentage-wise) we should
      // probably report some sort of generic error to the user.
      let input = ComplexNumber::new(x_values[idx.x].clone(), y_values[idx.y].clone());
      match z_function.eval_at(input, "real number", &ExprToNumber) {
        Err(_) => f64::NAN,
        Ok(res) => res.to_f64_or_nan(),
      }
    });

    let x_values: Vec<_> = floatify(x_values);
    let y_values: Vec<_> = floatify(y_values);

    Self { x_values, y_values, z_values }
  }
}
