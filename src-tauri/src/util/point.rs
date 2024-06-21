
//! Structs for manipulating points in 2D space.

use serde::{Serialize, Deserialize};

use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Point2D {
  pub x: f64,
  pub y: f64,
}

impl Display for Point2D {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "({}, {})", self.x, self.y)
  }
}
