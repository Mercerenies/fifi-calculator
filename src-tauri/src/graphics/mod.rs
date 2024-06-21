
//! Support for plotting and graphical output.

pub mod dataset;
pub mod payload;
pub mod plot;
pub mod response;

use crate::expr::number::Number;

use serde::{Serialize, Deserialize};

use std::borrow::Borrow;

/// Name of the function representing a 2D graphics object in the
/// expression language.
pub const GRAPHICS_NAME: &str = "graphics";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphicsType {
  #[serde(rename = "2D")]
  TwoDimensional,
}

impl GraphicsType {
  pub fn function_name(&self) -> &'static str {
    match self {
      GraphicsType::TwoDimensional => GRAPHICS_NAME,
    }
  }

  pub fn parse(name: &str) -> Option<Self> {
    if name == GRAPHICS_NAME {
      Some(GraphicsType::TwoDimensional)
    } else {
      None
    }
  }

  /// Returns true if `name` is a function name representing a
  /// graphics function. This function returns true if and only if
  /// [`GraphicsType::parse`] would succeed on the same input.
  pub fn is_graphics_function(name: &str) -> bool {
    GraphicsType::parse(name).is_some()
  }
}

/// Most of our computations are done in the [`Number`] struct. But
/// when we actually go to plot data, we use `f64`. This function
/// converts an iterable of `Number` values into an iterable of `f64`
/// values, replacing any numbers out of the range of `f64` with
/// `f64::NAN`.
pub fn floatify<I, T, C>(iter: I) -> C
where I: IntoIterator<Item = T>,
      T: Borrow<Number>,
      C: FromIterator<f64> {
  iter.into_iter().map(|n| n.borrow().to_f64_or_nan()).collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_graphics_type() {
    assert_eq!(GraphicsType::parse("graphics"), Some(GraphicsType::TwoDimensional));
    assert_eq!(GraphicsType::parse("xyzxyz"), None);
  }
}
