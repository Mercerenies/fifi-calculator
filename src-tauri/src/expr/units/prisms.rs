
use crate::util::prism::Prism;
use crate::units::unit::CompositeUnit;
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

#[derive(Debug, Clone)]
pub struct UnitPrism<'a, 'b, P: ?Sized, L: ?Sized, T> {
  parser: &'a P,
  language_mode: &'b L,
  _phantom: PhantomData<fn() -> T>,
}

impl<'a, 'b, P, L, T> UnitPrism<'a, 'b, P, L, T>
where P: UnitParser<T> + ?Sized,
      L: LanguageMode + ?Sized {
  pub fn new(parser: &'a P, language_mode: &'b L) -> Self {
    Self {
      parser,
      language_mode,
      _phantom: PhantomData,
    }
  }
}

impl<'a, 'b, P, L, T> Prism<String, ParsedCompositeUnit<T>> for UnitPrism<'a, 'b, P, L, T>
where P: UnitParser<T> + ?Sized,
      L: LanguageMode + ?Sized {
  fn narrow_type(&self, input: String) -> Result<ParsedCompositeUnit<T>, String> {
    let Ok(expr) = self.language_mode.parse(&input) else {
      return Err(input);
    };
    let tagged_expr = parse_composite_unit_expr(self.parser, expr);
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
