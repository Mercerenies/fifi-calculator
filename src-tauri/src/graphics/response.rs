
//! Response data for the `render_graphics` Tauri command.

use crate::util::point::Point2D;
use super::GraphicsType;

use serde::{Serialize, Deserialize};

use std::ops::Range;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GraphicsResponse {
  pub directives: Vec<GraphicsDirective>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GraphicsDirective {
  Plot(PlotDirective),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlotDirective {
  pub x_bounds: Range<f64>,
  pub y_bounds: Range<f64>,
  pub points: Vec<Point2D>,
}

impl GraphicsDirective {
  pub fn graphics_type(&self) -> GraphicsType {
    match self {
      GraphicsDirective::Plot(_) => GraphicsType::TwoDimensional,
    }
  }
}
