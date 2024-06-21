
//! Functionality for producing two-dimensional plots of data.

use crate::util::point::Point2D;
use crate::expr::number::Number;
use super::dataset::{XDataSet, LengthError};
use super::floatify;

use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlotDirective {
  pub points: Vec<Point2D>,
}

impl PlotDirective {
  pub fn empty() -> PlotDirective {
    PlotDirective { points: Vec::new() }
  }

  pub fn from_points(x_dataset: &XDataSet, y_points: &[Number]) -> Result<PlotDirective, LengthError> {
    let x_points = x_dataset.gen_points(Some(y_points.len()))?;
    assert!(x_points.len() == y_points.len(), "Expected two vectors of equal length");

    let x_points: Vec<_> = floatify(x_points);
    let y_points: Vec<_> = floatify(y_points);

    let points = x_points.into_iter().zip(y_points).map(|(x, y)| Point2D { x, y }).collect();

    Ok(PlotDirective {
      points,
    })
  }
}
