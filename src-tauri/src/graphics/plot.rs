
//! Functionality for producing two-dimensional plots of data.

use crate::util::point::Point2D;

use serde::{Serialize, Deserialize};

use std::ops::Range;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlotDirective {
  pub x_bounds: Range<f64>,
  pub y_bounds: Range<f64>,
  pub points: Vec<Point2D>,
}
