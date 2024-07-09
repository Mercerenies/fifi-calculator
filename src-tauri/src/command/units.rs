
//! Commands pertaining to unit arithmetic.

use super::arguments::{BinaryArgumentSchema, validate_schema};
use super::base::{Command, CommandContext, CommandOutput};
use super::functional::UnaryFunctionCommand;
use crate::state::ApplicationState;
use crate::display::language::LanguageMode;
use crate::expr::number::Number;
use crate::expr::algebra::term::Term;
use crate::expr::units::{try_parse_unit, UnitPrism, ParsedCompositeUnit};
use crate::units::parsing::UnitParser;

/// This command requires two arguments: the unit to convert from and
/// the unit to convert to. Both arguments are parsed with
/// [`UnitPrism`].
///
/// Converts the value on the top of the stack from the source unit
/// into the target unit. Does not attempt to parse the top of the
/// stack as a unital value, since both the source and target units
/// are being supplied externally.
///
/// This command always operates on the top value of the stack and
/// does not use the numerical argument. However, this command does
/// respect the "keep" modifier.
#[derive(Debug, Clone, Default)]
pub struct ConvertUnitsCommand {
  _priv: (),
}

impl ConvertUnitsCommand {
  pub fn new() -> Self {
    Self { _priv: () }
  }

/*
  fn argument_schema<'a, 'b>(
    state: &'a ApplicationState,
    context: &CommandContext<'_, 'b>,
  ) -> BinaryArgumentSchema<DynUnitPrism<'a, 'b>, ParsedCompositeUnit<Number>, DynUnitPrism<'a, 'b>, ParsedCompositeUnit<Number>> {
    let prism = UnitPrism::new(context.units_parser, state.display_settings().language_mode().as_ref());
    BinaryArgumentSchema::new(
      "valid unit expression".to_owned(),
      prism.clone(),
      "valid unit expression".to_owned(),
      prism,
    )
  }
*/
}

type DynUnitPrism<'a, 'b> = UnitPrism<'a, 'b, dyn UnitParser<Number>, dyn LanguageMode, Number>;

/// Unary command which removes units from the targeted stack
/// element(s).
pub fn remove_units_command() -> UnaryFunctionCommand {
  UnaryFunctionCommand::with_context(|arg, ctx| {
    // TODO: Evaluate if this is still valid if we add a mode that
    // treats multiplication as non-commutative.
    let term = Term::parse_expr(arg);
    let term = term.filter_factors(|expr| {
      try_parse_unit(ctx.units_parser, expr.to_owned()).is_err() // TODO: Excessive cloning
    });
    term.into()
  })
}

/// Unary command which keeps only the units from the targeted stack
/// element(s).
pub fn extract_units_command() -> UnaryFunctionCommand {
  UnaryFunctionCommand::with_context(|arg, ctx| {
    // TODO: Evaluate if this is still valid if we add a mode that
    // treats multiplication as non-commutative.
    let term = Term::parse_expr(arg);
    let term = term.filter_factors(|expr| {
      try_parse_unit(ctx.units_parser, expr.to_owned()).is_ok() // TODO: Excessive cloning
    });
    term.into()
  })
}

impl Command for ConvertUnitsCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    ctx: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    todo!()
  }
}
