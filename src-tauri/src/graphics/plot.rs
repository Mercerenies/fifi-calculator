
//! Functionality for producing two-dimensional plots of data.

use crate::util::point::Point2D;
use crate::util::prism::{Prism, OnTuple2};
use crate::expr::Expr;
use crate::expr::number::Number;
use crate::expr::algebra::ExprFunction;
use crate::expr::prisms::expr_to_number;
use super::dataset::{XDataSet, LengthError, GenReason};
use super::floatify;

use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlotDirective {
  pub points: Vec<Point2D>,
}

#[derive(Clone, Debug)]
pub struct ParametricResult {
  pub x_coord: Number,
  pub y_coord: Number,
}

#[derive(Clone, Debug, Copy, Default)]
pub struct ExprToParametricResult;

impl PlotDirective {
  pub fn empty() -> PlotDirective {
    PlotDirective { points: Vec::new() }
  }

  pub fn is_parametric_function(expr: &Expr) -> bool {
    let Expr::Call(function, args) = expr else {
      return false;
    };
    function == "xy" && args.len() == 2
  }

  pub fn from_points(x_dataset: &XDataSet, y_points: &[Number]) -> Result<PlotDirective, LengthError> {
    let x_points = x_dataset.gen_exact_points(Some(y_points.len()), GenReason::OneDimensional)?;
    assert!(x_points.len() == y_points.len(), "Expected two vectors of equal length");

    let x_points: Vec<_> = floatify(x_points);
    let y_points: Vec<_> = floatify(y_points);

    let points = x_points.into_iter().zip(y_points).map(|(x, y)| Point2D { x, y }).collect();
    Ok(PlotDirective {
      points,
    })
  }

  pub fn from_expr_function(x_dataset: &XDataSet, y_function: &ExprFunction) -> PlotDirective {
    let x_points = x_dataset.gen_points(GenReason::OneDimensional);
    let y_points = x_points.clone().into_iter().map(|x| {
      // TODO: If enough of these fail (percentage-wise) we should
      // probably report some sort of generic error to the user.
      match y_function.eval_at_real(x) {
        Ok(number) => number.to_f64_or_nan(),
        Err(_) => f64::NAN,
      }
    });

    let x_points: Vec<_> = floatify(x_points);

    let points = x_points.into_iter().zip(y_points).map(|(x, y)| Point2D { x, y }).collect();
    PlotDirective {
      points,
    }
  }

  pub fn from_parametric_function(t_dataset: &XDataSet, xy_function: &ExprFunction) -> PlotDirective {
    let t_points = t_dataset.gen_points(GenReason::OneDimensional);
    let xy_points = t_points.clone().into_iter().map(|t| {
      // TODO: If enough of these fail (percentage-wise) we should
      // probably report some sort of generic error to the user.
      match xy_function.eval_at(t, "Parametric result xy(_, _)", &ExprToParametricResult) {
        Ok(res) => Point2D { x: res.x_coord.to_f64_or_nan(), y: res.y_coord.to_f64_or_nan() },
        Err(_) => Point2D::NAN,
      }
    }).collect();

    PlotDirective {
      points: xy_points,
    }
  }
}

impl From<ParametricResult> for Point2D {
  fn from(parametric_result: ParametricResult) -> Point2D {
    Point2D {
      x: parametric_result.x_coord.to_f64_or_nan(),
      y: parametric_result.y_coord.to_f64_or_nan(),
    }
  }
}

impl Prism<Expr, ParametricResult> for ExprToParametricResult {
  fn narrow_type(&self, expr: Expr) -> Result<ParametricResult, Expr> {
    let Expr::Call(name, args) = expr else {
      return Err(expr);
    };
    if name != "xy" || args.len() != 2 {
      return Err(Expr::Call(name, args));
    }
    let [x, y] = args.try_into().unwrap();
    match OnTuple2::both(expr_to_number()).narrow_type((x, y)) {
      Err((x, y)) => Err(Expr::Call(name, vec![x, y])),
      Ok((x, y)) => Ok(ParametricResult { x_coord: x, y_coord: y }),
    }
  }
  fn widen_type(&self, result: ParametricResult) -> Expr {
    Expr::call("xy", vec![
      Expr::from(result.x_coord),
      Expr::from(result.y_coord),
    ])
  }
}
