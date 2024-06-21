
use crate::expr::{Expr, TryFromExprError};
use super::GraphicsType;

use serde::{Serialize, Deserialize};
use base64::engine::general_purpose::{STANDARD as BASE64_STANDARD};
use base64::Engine;

use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};
use std::io::Cursor;

/// A `GraphicsPayload` represents an function call expression whose
/// function is a graphics directive.
///
/// Currently, the only graphics directive is the 2D graphics
/// directive, simply called `graphics`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GraphicsPayload {
  arguments: Vec<Expr>,
  graphics_type: GraphicsType,
}

/// A `GraphicsPayload` serialized in CBOR and encoded in base64.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SerializedGraphicsPayload {
  base64: String,
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

impl SerializedGraphicsPayload {
  pub fn new(body: &GraphicsPayload) -> anyhow::Result<SerializedGraphicsPayload> {
    let mut bytes = Vec::<u8>::new();
    ciborium::into_writer(body, &mut bytes)?;
    Ok(SerializedGraphicsPayload {
      base64: BASE64_STANDARD.encode(&bytes),
    })
  }

  pub fn try_deserialize(self) -> anyhow::Result<GraphicsPayload> {
    let bytes = BASE64_STANDARD.decode(self.base64)?;
    let body = ciborium::from_reader(Cursor::new(bytes))?;
    Ok(body)
  }
}

impl Display for SerializedGraphicsPayload {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.write_str(&self.base64)
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::util::prism::ErrorWithPayload;

  #[test]
  fn test_is_graphics_directive_positive() {
    let expr = Expr::call("graphics", vec![]);
    assert!(GraphicsPayload::is_graphics_directive(&expr));
    let expr = Expr::call("graphics", vec![Expr::from(0)]);
    assert!(GraphicsPayload::is_graphics_directive(&expr));
    let expr = Expr::call("graphics", vec![Expr::from(0), Expr::call("xyz", vec![])]);
    assert!(GraphicsPayload::is_graphics_directive(&expr));
  }

  #[test]
  fn test_is_graphics_directive_negative() {
    let expr = Expr::from(99);
    assert!(!GraphicsPayload::is_graphics_directive(&expr));
    let expr = Expr::call("foobar", vec![]);
    assert!(!GraphicsPayload::is_graphics_directive(&expr));
    let expr = Expr::call("foobar", vec![Expr::from(0)]);
    assert!(!GraphicsPayload::is_graphics_directive(&expr));
    let expr = Expr::call("GrApHiCs", vec![]);
    assert!(!GraphicsPayload::is_graphics_directive(&expr));
  }

  #[test]
  fn test_parse_graphics_directive() {
    let expr = Expr::call("graphics", vec![Expr::from(0)]);
    let graphics_payload = GraphicsPayload::try_from(expr.clone()).unwrap();
    assert_eq!(Expr::from(graphics_payload), expr);
  }

  #[test]
  fn test_parse_graphics_directive_failure() {
    let expr = Expr::call("foobar", vec![Expr::from(0)]);
    let err = GraphicsPayload::try_from(expr.clone()).unwrap_err();
    assert_eq!(err.recover_payload(), expr);
  }
}
