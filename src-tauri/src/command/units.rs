
//! Commands pertaining to unit arithmetic.

use super::arguments::{UnaryArgumentSchema, BinaryArgumentSchema, validate_schema};
use super::base::{Command, CommandContext, CommandOutput};
use super::functional::UnaryFunctionCommand;
use crate::errorlist::ErrorList;
use crate::state::ApplicationState;
use crate::stack::base::StackLike;
use crate::stack::keepable::KeepableStack;
use crate::display::language::LanguageMode;
use crate::expr::Expr;
use crate::expr::number::Number;
use crate::expr::algebra::term::Term;
use crate::expr::simplifier::{Simplifier, SimplifierContext};
use crate::expr::simplifier::chained::ChainedSimplifier;
use crate::expr::units::{parse_composite_unit_expr, try_parse_unit,
                         unit_into_term, tagged_into_expr,
                         UnitPrism, ParsedCompositeUnit,
                         UnitTermSimplifier};
use crate::units::CompositeUnit;
use crate::units::parsing::UnitParser;
use crate::units::tagged::Tagged;
use crate::units::dimension::Dimension;

use anyhow::Context;

/// This command requires two arguments: the unit to convert from and
/// the unit to convert to. Both arguments are parsed with
/// [`UnitPrism`].
///
/// Converts the value on the top of the stack from the source unit
/// into the target unit. Does not attempt to parse the top of the
/// stack as a unital value, since both the source and target units
/// are being supplied externally. If the dimensions of the source and
/// target units do not match, remainder units will be inserted to
/// make the dimensions match.
///
/// Aside from remainder units, no units are inserted into the
/// resulting stack expression.
///
/// This command always operates on the top value of the stack and
/// does not use the numerical argument. However, this command does
/// respect the "keep" modifier.
#[derive(Debug, Clone, Default)]
pub struct ConvertUnitsCommand {
  _priv: (),
}

/// This command requires one argument: the target unit. The target
/// unit will be parsed via [`UnitPrism`].
///
/// Pops the top value of the stack, interpreting it as an expression
/// with units already present, and converts that expression into the
/// given target unit. If the dimensions do not match, then remainder
/// units will be inserted.
///
/// The new units will be present on the top stack element when this
/// operation is done.
///
/// This command always operates on the top value of the stack and
/// does not use the numerical argument. However, this command does
/// respect the "keep" modifier.
#[derive(Debug, Clone, Default)]
pub struct ContextualConvertUnitsCommand {
  _priv: (),
}

impl ConvertUnitsCommand {
  pub fn new() -> Self {
    Self { _priv: () }
  }

  fn argument_schema<'p, 'm>(
    state: &'m ApplicationState,
    context: &CommandContext<'_, 'p>,
  ) -> BinaryArgumentSchema<ConcreteUnitPrism<'p, 'm>, ParsedCompositeUnit<Number>, ConcreteUnitPrism<'p, 'm>, ParsedCompositeUnit<Number>> {
    BinaryArgumentSchema::new(
      "valid unit expression".to_owned(),
      UnitPrism::new(context.units_parser, state.display_settings().language_mode()),
      "valid unit expression".to_owned(),
      UnitPrism::new(context.units_parser, state.display_settings().language_mode()),
    )
  }
}

impl ContextualConvertUnitsCommand {
  pub fn new() -> Self {
    Self { _priv: () }
  }

  fn argument_schema<'p, 'm>(
    state: &'m ApplicationState,
    context: &CommandContext<'_, 'p>,
  ) -> UnaryArgumentSchema<ConcreteUnitPrism<'p, 'm>, ParsedCompositeUnit<Number>> {
    UnaryArgumentSchema::new(
      "valid unit expression".to_owned(),
      UnitPrism::new(context.units_parser, state.display_settings().language_mode()),
    )
  }
}

type ConcreteUnitPrism<'p, 'm> = UnitPrism<&'p dyn UnitParser<Number>, Box<dyn LanguageMode + 'm>, Number>;

fn calculate_remainder_unit<P>(parser: &P, source_dim: &Dimension, target_dim: &Dimension) -> CompositeUnit<Number>
where P: UnitParser<Number> + ?Sized {
  let remainder_dim = source_dim.to_owned() / target_dim.to_owned();
  parser.base_composite_unit(&remainder_dim)
}

/// Simplifier which runs a unit simplification step after the usual
/// simplification step.
fn unit_simplifier<'a>(ctx: &'a CommandContext) -> ChainedSimplifier<'a, 'a> {
  ChainedSimplifier::new(
    Box::new(ctx.simplifier.as_ref()),
    Box::new(UnitTermSimplifier::new(ctx.units_parser)),
  )
}

/// Unary command which simplifies units on the targeted stack
/// element(s).
pub fn simplify_units_command() -> UnaryFunctionCommand {
  UnaryFunctionCommand::with_context_and_errors(|arg, ctx, errors| {
    let simplifier = unit_simplifier(ctx);
    let mut simplifier_ctx = SimplifierContext {
      base_simplifier: &simplifier,
      errors,
    };
    simplifier.simplify_expr(arg, &mut simplifier_ctx)
  })
}

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
    let (source_unit, target_unit) = validate_schema(&Self::argument_schema(state, ctx), args)?;
    let source_unit = CompositeUnit::from(source_unit);
    let target_unit = CompositeUnit::from(target_unit);

    let remainder_unit = calculate_remainder_unit(
      ctx.units_parser,
      &source_unit.dimension(),
      &target_unit.dimension(),
    );
    let remainder_term = unit_into_term(remainder_unit.clone())
      .context("Remainder unit contained an invalid variable name")?; // TODO: Restore the stack on error?

    state.undo_stack_mut().push_cut();
    let mut stack = KeepableStack::new(state.main_stack_mut(), ctx.opts.keep_modifier);
    let term = Term::parse_expr(stack.pop()?);
    let term = Tagged::new(term, source_unit);

    // convert_or_panic safety: We already forced the dimensions to
    // line up, using the remainder unit.
    let term = term.convert_or_panic(target_unit * remainder_unit);
    let term = term.value * remainder_term;
    let expr = Expr::from(term);

    let mut errors = ErrorList::new();
    stack.push(ctx.simplify_expr(expr, &mut errors));
    Ok(CommandOutput::from_errors(errors))
  }
}

impl Command for ContextualConvertUnitsCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    ctx: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    let target_unit = validate_schema(&Self::argument_schema(state, ctx), args)?;
    let target_unit = CompositeUnit::from(target_unit);

    state.undo_stack_mut().push_cut();
    let mut stack = KeepableStack::new(state.main_stack_mut(), ctx.opts.keep_modifier);
    let tagged_term = parse_composite_unit_expr(ctx.units_parser, stack.pop()?);

    let remainder_unit = calculate_remainder_unit(
      ctx.units_parser,
      &tagged_term.unit.dimension(),
      &target_unit.dimension(),
    );

    // convert_or_panic safety: We already forced the dimensions to
    // line up, using the remainder unit.
    let tagged_term = tagged_term.convert_or_panic(target_unit * remainder_unit);
    let expr = tagged_into_expr(tagged_term)
      .context("Target unit contained an invalid variable name")?; // TODO: Restore the stack on error?

    let mut errors = ErrorList::new();
    stack.push(ctx.simplify_expr(expr, &mut errors));
    Ok(CommandOutput::from_errors(errors))
  }
}

#[cfg(test)]
pub(crate) mod test_utils {
  use super::*;
  use crate::command::CommandContext;
  use crate::units::parsing::{default_parser, PrefixParser, TableBasedParser};

  use once_cell::sync::Lazy;

  /// This function is an
  /// [`ActOnStackArg`](crate::command::test_utils::ActOnStackArg)
  /// which sets up the SI units table on the current command context.
  pub fn setup_si_units(_args: &mut Vec<String>, context: &mut CommandContext) {
    static PARSER: Lazy<PrefixParser<TableBasedParser<Number>>> = Lazy::new(default_parser);
    let concrete_parser = Lazy::force(&PARSER);
    context.units_parser = concrete_parser;
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use super::test_utils::setup_si_units;
  use crate::command::test_utils::{act_on_stack, setup_default_simplifier};
  use crate::command::options::CommandOptions;
  use crate::stack::test_utils::stack_of;

  #[test]
  fn test_simple_length_conversion_down() {
    let output_stack = act_on_stack(
      &ConvertUnitsCommand::new(),
      (setup_si_units, setup_default_simplifier, vec!["m", "cm"]),
      vec![10, 20, 30, 1_000],
    ).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 100_000]));
  }

  #[test]
  fn test_simple_length_conversion_up() {
    let output_stack = act_on_stack(
      &ConvertUnitsCommand::new(),
      (setup_si_units, setup_default_simplifier, vec!["cm", "m"]),
      vec![10, 20, 30, 1_000],
    ).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 10]));
  }

  #[test]
  fn test_simple_length_conversion_with_keep_modifier() {
    let setup = (setup_si_units, setup_default_simplifier);
    let output_stack = act_on_stack(
      &ConvertUnitsCommand::new(),
      (setup, CommandOptions::default().with_keep_modifier(), vec!["cm", "m"]),
      vec![10, 20, 30, 1_000],
    ).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 1_000, 10]));
  }

  #[test]
  fn test_area_conversion() {
    let output_stack = act_on_stack(
      &ConvertUnitsCommand::new(),
      (setup_si_units, setup_default_simplifier, vec!["m^2", "km^2"]),
      vec![10, 20, 30, 1_000_000],
    ).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 1]));
  }

  #[test]
  fn test_area_conversion_fractional() {
    let output_stack = act_on_stack(
      &ConvertUnitsCommand::new(),
      (setup_si_units, setup_default_simplifier, vec!["m^2", "km^2"]),
      vec![1_000],
    ).unwrap();
    assert_eq!(output_stack, stack_of(vec![Number::ratio(1, 1_000)]));
  }

  #[test]
  fn test_nontrivial_dimension_conversion() {
    let output_stack = act_on_stack(
      &ConvertUnitsCommand::new(),
      (setup_si_units, setup_default_simplifier, vec!["m / s", "mph"]),
      vec![600],
    ).unwrap();
    assert_eq!(output_stack, stack_of(vec![Number::ratio(1_875_000, 1_397)]));
  }

  #[test]
  fn test_conversion_with_remainder() {
    let output_stack = act_on_stack(
      &ConvertUnitsCommand::new(),
      (setup_si_units, setup_default_simplifier, vec!["m / s", "km"]),
      vec![3],
    ).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::call("/", vec![
        Expr::from(3),
        Expr::call("*", vec![
          Expr::from(1_000),
          Expr::var("s").unwrap(),
        ]),
      ]),
    ]));
  }

  #[test]
  fn test_simple_length_context_conversion() {
    let setup = (setup_si_units, setup_default_simplifier);
    let input_stack = vec![
      Expr::call("*", vec![
        Expr::from(100),
        Expr::var("cm").unwrap(),
      ])
    ];
    let output_stack = act_on_stack(
      &ContextualConvertUnitsCommand::new(),
      (setup, vec!["m"]),
      input_stack,
    ).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::call("*", vec![
        Expr::from(100),
        Expr::from(Number::ratio(1, 100)),
        Expr::var("m").unwrap(),
      ]),
    ]));
  }

  #[test]
  fn test_simple_length_context_conversion_with_keep_modifier() {
    let setup = (setup_si_units, setup_default_simplifier);
    let input_stack = vec![
      Expr::call("*", vec![
        Expr::from(100),
        Expr::var("cm").unwrap(),
      ])
    ];
    let output_stack = act_on_stack(
      &ContextualConvertUnitsCommand::new(),
      (setup, CommandOptions::default().with_keep_modifier(), vec!["m"]),
      input_stack,
    ).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::call("*", vec![
        Expr::from(100),
        Expr::var("cm").unwrap(),
      ]),
      Expr::call("*", vec![
        Expr::from(100),
        Expr::from(Number::ratio(1, 100)),
        Expr::var("m").unwrap(),
      ]),
    ]));
  }

  #[test]
  fn test_context_conversion_with_remainder_dimension() {
    let setup = (setup_si_units, setup_default_simplifier);
    let input_stack = vec![
      Expr::call("*", vec![
        Expr::from(100),
        Expr::call("/", vec![
          Expr::var("cm").unwrap(),
          Expr::var("sec").unwrap(),
        ]),
      ])
    ];
    let output_stack = act_on_stack(
      &ContextualConvertUnitsCommand::new(),
      (setup, vec!["m"]),
      input_stack,
    ).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::call("/", vec![
        Expr::call("*", vec![
          Expr::from(100),
          Expr::from(Number::ratio(1, 100)),
          Expr::var("m").unwrap(),
        ]),
        Expr::var("s").unwrap(),
      ]),
    ]));
  }
}
