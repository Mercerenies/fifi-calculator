
use crate::util::prism::Prism;
use crate::units::CompositeUnit;
use crate::units::parsing::UnitParser;
use crate::display::language::LanguageMode;
use super::parser::parse_composite_unit_expr;

use num::One;

use std::marker::PhantomData;

/// A composite unit tagged with the string that produced it.
pub struct ParsedCompositeUnit<T> {
  pub unit: CompositeUnit<T>,
  pub original_string: String,
}

#[derive(Debug)]
pub struct UnitPrism<P, L, T> {
  parser: P,
  language_mode: L,
  _phantom: PhantomData<fn() -> T>,
}

impl<P, L, T> UnitPrism<P, L, T>
where P: UnitParser<T>,
      L: LanguageMode {
  pub fn new(parser: P, language_mode: L) -> Self {
    Self {
      parser,
      language_mode,
      _phantom: PhantomData,
    }
  }
}

impl<P, L, T> Clone for UnitPrism<P, L, T>
where P: UnitParser<T> + Clone,
      L: LanguageMode + Clone {
  fn clone(&self) -> Self {
    Self {
      parser: self.parser.clone(),
      language_mode: self.language_mode.clone(),
      _phantom: PhantomData,
    }
  }
}

impl<P, L, T> Prism<String, ParsedCompositeUnit<T>> for UnitPrism<P, L, T>
where P: UnitParser<T>,
      L: LanguageMode {
  fn narrow_type(&self, input: String) -> Result<ParsedCompositeUnit<T>, String> {
    let Ok(expr) = self.language_mode.parse(&input) else {
      return Err(input);
    };
    let tagged_expr = parse_composite_unit_expr(&self.parser, expr);
    if tagged_expr.value.is_one() {
      Ok(ParsedCompositeUnit {
        unit: tagged_expr.unit,
        original_string: input,
      })
    } else {
      Err(input)
    }
  }

  fn widen_type(&self, input: ParsedCompositeUnit<T>) -> String {
    input.original_string
  }
}

impl<T> From<ParsedCompositeUnit<T>> for CompositeUnit<T> {
  fn from(parsed: ParsedCompositeUnit<T>) -> Self {
    parsed.unit
  }
}
