
//! Support for plotting and graphical output.

use crate::expr::{Expr, TryFromExprError};

use serde::{Serialize, Deserialize};

use std::convert::TryFrom;

/// Name of the function representing a 2D graphics object in the
/// expression language.
pub const GRAPHICS_NAME: &str = "graphics";

/// A `GraphicsPayload` represents an function call expression whose
/// function is a graphics directive.
///
/// Currently, the only graphics directive is the 2D graphics
/// directive, simply called `graphics`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphicsPayload {
  arguments: Vec<Expr>,
  graphics_type: GraphicsType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphicsType {
  #[serde(rename = "2D")]
  TwoDimensional,
}

impl GraphicsPayload {
  pub fn new(arguments: Vec<Expr>, graphics_type: GraphicsType) -> Self {
    Self { arguments, graphics_type }
  }

  pub fn graphics_type(&self) -> GraphicsType {
    self.graphics_type
  }

  pub fn arguments(&self) -> &[Expr] {
    &self.arguments
  }

  pub fn into_args(self) -> Vec<Expr> {
    self.arguments
  }

  /// Returns true if the expression can be parsed as a
  /// [`GraphicsPayload`], without consuming the expression. This
  /// function returns true if and only if
  /// `GraphicsPayload::try_from(expr)` would succeed.
  pub fn is_graphics_directive(expr: &Expr) -> bool {
    if let Expr::Call(name, _) = &expr {
      GraphicsType::is_graphics_function(name)
    } else {
      false
    }
  }
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

impl From<GraphicsPayload> for Expr {
  fn from(payload: GraphicsPayload) -> Self {
    Expr::call(payload.graphics_type.function_name(), payload.arguments)
  }
}

impl TryFrom<Expr> for GraphicsPayload {
  type Error = TryFromExprError;

  fn try_from(expr: Expr) -> Result<Self, Self::Error> {
    if let Expr::Call(name, args) = expr {
      if let Some(graphics_type) = GraphicsType::parse(&name) {
        Ok(GraphicsPayload::new(args, graphics_type))
      } else {
        Err(TryFromExprError::new("GraphicsPayload", Expr::Call(name, args)))
      }
    } else {
      Err(TryFromExprError::new("GraphicsPayload", expr))
    }
  }
}
