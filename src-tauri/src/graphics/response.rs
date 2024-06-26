
//! Response data for the `render_graphics` Tauri command.

use super::GraphicsType;
use super::plot::PlotDirective;
use super::contour_plot::ContourPlotDirective;

use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct GraphicsResponse {
  pub directives: Vec<GraphicsDirective>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GraphicsDirective {
  #[serde(rename = "plot")]
  Plot(PlotDirective),
  #[serde(rename = "contourplot")]
  ContourPlot(ContourPlotDirective),
}

impl GraphicsDirective {
  pub fn graphics_type(&self) -> GraphicsType {
    match self {
      GraphicsDirective::Plot(_) => GraphicsType::TwoDimensional,
      GraphicsDirective::ContourPlot(_) => GraphicsType::TwoDimensional,
    }
  }
}

impl GraphicsResponse {
  pub fn new() -> Self {
    Self::default()
  }
}
