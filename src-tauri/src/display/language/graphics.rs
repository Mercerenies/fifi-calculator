
use super::{LanguageMode, LanguageModeEngine};
use crate::expr::Expr;
use crate::parsing::operator::Precedence;
use crate::graphics::payload::GraphicsPayload;

use base64::engine::general_purpose::{STANDARD as BASE64_STANDARD};
use base64::Engine;

use std::convert::TryFrom;
use std::fmt::Write;

/// The `GraphicsLanguageMode` wraps an existing `LanguageMode` and
/// adds functionality to support the graphics subsystem.
///
/// Specifically, whenever a graphics directive is encountered (per
/// [`GraphicsPayload::is_graphics_directive`]), the output for the
/// directive is wrapped in an HTML `<span>` which contains the
/// graphics payload in a data attribute.
///
/// The `GraphicsLanguageMode` does not modify the inner language
/// mode's parsing capabilities. `<GraphicsLanguageMode as
/// LanguageMode>::parse` simply delegates to the inner implementation
/// without any changes.
pub struct GraphicsLanguageMode<'a, L: LanguageMode + ?Sized> {
  inner: &'a L,
}

impl<'a, L: LanguageMode + ?Sized> GraphicsLanguageMode<'a, L> {
  pub fn new(inner: &'a L) -> GraphicsLanguageMode<'a, L> {
    GraphicsLanguageMode { inner }
  }

  fn write_graphics_payload(&self, out: &mut String, payload: &GraphicsPayload) {
    let payload = {
      let mut bytes = Vec::<u8>::new();
      ciborium::into_writer(payload, &mut bytes).expect("Failed to serialize graphics payload");
      BASE64_STANDARD.encode(&bytes)
    };
    write!(out, r#"<span data-graphics-flag="true" data-graphics-payload="{}">"#, payload)
      .expect("Failed to write graphics payload to string");
  }
}

impl<'a, L: LanguageMode + ?Sized> LanguageMode for GraphicsLanguageMode<'a, L> {
  fn write_to_html(&self, engine: &LanguageModeEngine, out: &mut String, expr: &Expr, prec: Precedence) {
    let is_graphics_directive = GraphicsPayload::is_graphics_directive(expr);
    if is_graphics_directive {
      let expr = expr.to_owned();
      let payload = GraphicsPayload::try_from(expr).expect("Failed to parse graphics directive");
      self.write_graphics_payload(out, &payload);
    }
    self.inner.write_to_html(engine, out, expr, prec);
    if is_graphics_directive {
      out.push_str("</span>");
    }
  }

  fn parse(&self, text: &str) -> anyhow::Result<Expr> {
    self.inner.parse(text)
  }

  fn to_trait_object(&self) -> &dyn LanguageMode {
    self
  }
}
