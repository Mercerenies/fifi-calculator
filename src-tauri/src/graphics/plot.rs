
//! Functionality for producing two-dimensional plots of data.

use crate::util::point::Point2D;
use crate::expr::number::Number;
use super::dataset::{XDataSet, LengthError};
use super::floatify;

use serde::{Serialize, Deserialize};

use std::ops::Range;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlotDirective {
  pub x_bounds: Range<f64>,
  pub y_bounds: Range<f64>,
  pub points: Vec<Point2D>,
}

impl PlotDirective {
  pub fn empty() -> PlotDirective {
    PlotDirective { x_bounds: 0.0..1.0, y_bounds: 0.0..1.0, points: Vec::new() }
  }

  pub fn from_points(x_dataset: &XDataSet, y_points: &[Number]) -> Result<PlotDirective, LengthError> {
    let x_points = x_dataset.gen_points(Some(y_points.len()))?;
    assert!(x_points.len() == y_points.len(), "Expected two vectors of equal length");

    if y_points.is_empty() {
      return Ok(PlotDirective::empty());
    }

    let x_points: Vec<_> = floatify(x_points);
    let y_points: Vec<_> = floatify(y_points);

    // unwrap: We handled the empty case at the very beginning.
    let x_min = x_points.iter().copied().reduce(f64::min).unwrap() - 1.0;
    let x_max = x_points.iter().copied().reduce(f64::max).unwrap() + 1.0;
    let y_min = y_points.iter().copied().reduce(f64::min).unwrap() - 1.0;
    let y_max = y_points.iter().copied().reduce(f64::max).unwrap() + 1.0;
    let points = x_points.into_iter().zip(y_points).map(|(x, y)| Point2D { x, y }).collect();

    Ok(PlotDirective {
      x_bounds: x_min..x_max,
      y_bounds: y_min..y_max,
      points,
    })
  }
}
