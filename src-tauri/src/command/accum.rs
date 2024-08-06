
//! Commands which reduce or operate over vectors, usually using a
//! subcommand in a higher-order capacity.
//!
//! For first-order vector commands, see the [`vector`
//! module](crate::command::vector).

use super::arguments::{UnaryArgumentSchema, BinaryArgumentSchema, validate_schema};
use super::base::{Command, CommandContext, CommandOutput};
use super::options::CommandOptions;
use super::subcommand::{Subcommand, StringToSubcommandId, ParsedSubcommandId};
use crate::util;
use crate::util::prism::{Prism, PrismExt};
use crate::errorlist::ErrorList;
use crate::expr::prisms;
use crate::expr::vector::Vector;
use crate::state::ApplicationState;
use crate::stack::base::StackLike;
use crate::stack::keepable::KeepableStack;

/// `VectorApplyCommand` expects a subcommand as argument. This
/// command pops a single value off the stack, which must be a vector,
/// and applies the subcommand with the vector elements as arguments.
/// If the vector contains the wrong number of elements, an error is
/// signaled. Does not use the numerical argument, but respects the
/// "keep" modifier.
#[derive(Debug, Default)]
pub struct VectorApplyCommand {
  _priv: (),
}

/// `VectorMapCommand` expects a unary subcommand as argument. This
/// command pops a single value off the stack, which must be a vector,
/// and applies the subcommand to each vector element separately,
/// producing a new vector.
///
/// Respects the "keep" modifier.
#[derive(Debug, Default)]
pub struct VectorMapCommand {
  _priv: (),
}

/// `VectorReduceCommand` expects a binary subcommand as argument.
/// This command pops a single value off the stack, which must be a
/// nonempty vector. The subcommand is used to reduce the vector and
/// produce a single scalar.
///
/// Respects the "keep" modifier.
#[derive(Debug)]
pub struct VectorReduceCommand {
  direction: ReduceDir,
}

/// `VectorAccumCommand` works similarly to [`VectorReduceCommand`] but
/// produces a vector of intermediate values rather than a single
/// scalar result.
///
/// Specifically, `VectorAccumCommand` expects a binary subcommand as
/// argument. This command pops a single value off the stack, which
/// must be a vector. The subcommand is used to reduce the vector, and
/// a resulting vector of the same length (containing the intermediate
/// results) is pushed onto the stack.
///
/// Respects the "keep" modifier.
#[derive(Debug)]
pub struct VectorAccumCommand {
  direction: ReduceDir,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReduceDir {
  LeftToRight,
  RightToLeft,
}

/// `OuterProductCommand` expects a binary subcommand as argument.
/// This command pops two values off the stack, both of which shall be
/// vectors. The subcommand is applied to every possible combination
/// of one element from the first vector and one element from the
/// second, producing a matrix of results, where each row represents a
/// value from the first vector and each column represents a value
/// from the second vector.
///
/// Respects the "keep" modifier.
#[derive(Debug)]
pub struct OuterProductCommand {
  _priv: (),
}

/// `InnerProductCommand` expects a pair of binary subcommands as
/// arguments. The first will be treated as a multiplication operation
/// and the second as addition.
///
/// This command pops two values off the stack, which shall be
/// non-empty vectors of the same length. The vectors are paired off
/// pointwise and multiplied together according to the first
/// subcommand. Then the results of such multiplication are reduced
/// (from the left) using the additive subcommand to produce a single
/// scalar as the result. This scalar is pushed onto the stack.
///
/// Respects the "keep" modifier.
#[derive(Debug)]
pub struct InnerProductCommand {
  _priv: (),
}

fn unary_subcommand_argument_schema() -> UnaryArgumentSchema<StringToSubcommandId, ParsedSubcommandId> {
  UnaryArgumentSchema::new(
    "subcommand identifier".to_string(),
    StringToSubcommandId,
  )
}

fn binary_subcommand_argument_schema() -> BinaryArgumentSchema<StringToSubcommandId, ParsedSubcommandId, StringToSubcommandId, ParsedSubcommandId> {
  BinaryArgumentSchema::new(
    "subcommand identifier".to_string(),
    StringToSubcommandId,
    "subcommand identifier".to_string(),
    StringToSubcommandId,
  )
}

impl VectorApplyCommand {
  pub fn new() -> Self {
    Self { _priv: () }
  }
}

impl VectorMapCommand {
  pub fn new() -> Self {
    Self { _priv: () }
  }
}

impl VectorReduceCommand {
  pub fn new(direction: ReduceDir) -> Self {
    Self { direction }
  }

  pub fn direction(&self) -> ReduceDir {
    self.direction
  }
}

impl VectorAccumCommand {
  pub fn new(direction: ReduceDir) -> Self {
    Self { direction }
  }

  pub fn direction(&self) -> ReduceDir {
    self.direction
  }
}

impl OuterProductCommand {
  pub fn new() -> Self {
    Self { _priv: () }
  }
}

impl InnerProductCommand {
  pub fn new() -> Self {
    Self { _priv: () }
  }
}

impl Command for VectorApplyCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    context: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    let subcommand_id = validate_schema(&unary_subcommand_argument_schema(), args)?;
    let calculation_mode = state.calculation_mode().clone();
    let mut errors = ErrorList::new();
    let simplifier = context.simplifier.as_ref();
    state.undo_stack_mut().push_cut();
    let mut stack = KeepableStack::new(state.main_stack_mut(), context.opts.keep_modifier);

    let subcommand = subcommand_id.as_ref().get_subcommand(context.dispatch_table)?;
    let input_expr = stack.pop()?;
    let vec = match prisms::ExprToVector.narrow_type(input_expr) {
      Ok(vec) => vec,
      Err(input_expr) => {
        if !context.opts.keep_modifier {
          stack.push(input_expr);
        }
        anyhow::bail!("Expected vector");
      }
    };
    let expr = match subcommand.try_call(Vec::from(vec), simplifier, calculation_mode, &mut errors) {
      Ok(expr) => expr,
      Err(err) => {
        if !context.opts.keep_modifier {
          stack.push(prisms::ExprToVector.widen_type(Vector::from(err.args.clone())));
        }
        return Err(err.into());
      }
    };
    stack.push(expr);
    Ok(CommandOutput::from_errors(errors))
  }

  fn as_subcommand(&self, _opts: &CommandOptions) -> Option<Subcommand> {
    None
  }
}

impl Command for VectorMapCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    context: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    let subcommand_id = validate_schema(&unary_subcommand_argument_schema(), args)?;
    let calculation_mode = state.calculation_mode().clone();
    let mut errors = ErrorList::new();
    let simplifier = context.simplifier.as_ref();
    state.undo_stack_mut().push_cut();
    let mut stack = KeepableStack::new(state.main_stack_mut(), context.opts.keep_modifier);

    let subcommand = subcommand_id.as_ref().get_subcommand(context.dispatch_table)?;
    anyhow::ensure!(subcommand.arity() == 1, "Expected unary subcommand");

    let input_expr = stack.pop()?;
    let vec = match prisms::ExprToVector.narrow_type(input_expr) {
      Ok(vec) => vec,
      Err(input_expr) => {
        if !context.opts.keep_modifier {
          stack.push(input_expr);
        }
        anyhow::bail!("Expected vector");
      }
    };
    // call_or_panic: We checked the arity above.
    let output_vec: Vector = vec.into_iter()
      .map(|expr| subcommand.call_or_panic(vec![expr], simplifier, calculation_mode.clone(), &mut errors))
      .collect();
    stack.push(output_vec.into());
    Ok(CommandOutput::from_errors(errors))
  }

  fn as_subcommand(&self, _opts: &CommandOptions) -> Option<Subcommand> {
    None
  }
}

impl Command for VectorReduceCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    context: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    let subcommand_id = validate_schema(&unary_subcommand_argument_schema(), args)?;
    let calculation_mode = state.calculation_mode().clone();
    let mut errors = ErrorList::new();
    let simplifier = context.simplifier.as_ref();
    state.undo_stack_mut().push_cut();
    let mut stack = KeepableStack::new(state.main_stack_mut(), context.opts.keep_modifier);

    let subcommand = subcommand_id.as_ref().get_subcommand(context.dispatch_table)?;
    anyhow::ensure!(subcommand.arity() == 2, "Expected binary subcommand");

    let input_expr = stack.pop()?;
    let vec = match prisms::ExprToVector.narrow_type(input_expr) {
      Ok(vec) => vec,
      Err(input_expr) => {
        if !context.opts.keep_modifier {
          stack.push(input_expr);
        }
        anyhow::bail!("Expected vector");
      }
    };
    if vec.is_empty() {
      if !context.opts.keep_modifier {
        stack.push(prisms::ExprToVector.widen_type(vec));
      }
      anyhow::bail!("Expected non-empty vector");
    }

    // call_or_panic: We checked the arity above.
    let expr = match self.direction {
      ReduceDir::LeftToRight => {
        vec.into_iter().reduce(|a, b| {
          subcommand.call_or_panic(vec![a, b], simplifier, calculation_mode.clone(), &mut errors)
        }).unwrap() // unwrap: Vector is non-empty
      }
      ReduceDir::RightToLeft => {
        util::reduce_right(vec.into_iter(), |a, b| {
          subcommand.call_or_panic(vec![a, b], simplifier, calculation_mode.clone(), &mut errors)
        }).unwrap() // unwrap: Vector is non-empty
      }
    };
    stack.push(expr);
    Ok(CommandOutput::from_errors(errors))
  }

  fn as_subcommand(&self, _opts: &CommandOptions) -> Option<Subcommand> {
    None
  }
}

impl Command for VectorAccumCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    context: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    let subcommand_id = validate_schema(&unary_subcommand_argument_schema(), args)?;
    let calculation_mode = state.calculation_mode().clone();
    let mut errors = ErrorList::new();
    let simplifier = context.simplifier.as_ref();
    state.undo_stack_mut().push_cut();
    let mut stack = KeepableStack::new(state.main_stack_mut(), context.opts.keep_modifier);

    let subcommand = subcommand_id.as_ref().get_subcommand(context.dispatch_table)?;
    anyhow::ensure!(subcommand.arity() == 2, "Expected binary subcommand");

    let input_expr = stack.pop()?;
    let vec = match prisms::ExprToVector.narrow_type(input_expr) {
      Ok(vec) => vec,
      Err(input_expr) => {
        if !context.opts.keep_modifier {
          stack.push(input_expr);
        }
        anyhow::bail!("Expected vector");
      }
    };

    // call_or_panic: We checked the arity above.
    let output_vec = match self.direction {
      ReduceDir::LeftToRight => {
        util::accum_left(vec.into_iter(), |a, b| {
          subcommand.call_or_panic(vec![a, b], simplifier, calculation_mode.clone(), &mut errors)
        }).collect::<Vector>()
      }
      ReduceDir::RightToLeft => {
        let mut output_vec = util::accum_right(vec.into_iter(), |a, b| {
          subcommand.call_or_panic(vec![a, b], simplifier, calculation_mode.clone(), &mut errors)
        }).collect::<Vector>();
        output_vec.as_mut_vec().reverse();
        output_vec
      }
    };
    stack.push(output_vec.into());
    Ok(CommandOutput::from_errors(errors))
  }

  fn as_subcommand(&self, _opts: &CommandOptions) -> Option<Subcommand> {
    None
  }
}

impl Command for OuterProductCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    context: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    let subcommand_id = validate_schema(&unary_subcommand_argument_schema(), args)?;
    let calculation_mode = state.calculation_mode().clone();
    let mut errors = ErrorList::new();
    let simplifier = context.simplifier.as_ref();
    state.undo_stack_mut().push_cut();
    let mut stack = KeepableStack::new(state.main_stack_mut(), context.opts.keep_modifier);

    let subcommand = subcommand_id.as_ref().get_subcommand(context.dispatch_table)?;
    anyhow::ensure!(subcommand.arity() == 2, "Expected binary subcommand");

    let prism = prisms::ExprToVector.and(prisms::ExprToVector);

    let [a_vec, b_vec] = stack.pop_several(2)?.try_into().unwrap();
    let (a_vec, b_vec) = match prism.narrow_type((a_vec, b_vec)) {
      Ok(values) => values,
      Err((a_vec, b_vec)) => {
        if !context.opts.keep_modifier {
          stack.push_several([a_vec, b_vec]);
        }
        anyhow::bail!("Expected two vectors");
      }
    };

    // call_or_panic: We checked the arity above.
    let output_matrix = a_vec.outer_product(b_vec, |a, b| {
      subcommand.call_or_panic(vec![a, b], simplifier, calculation_mode.clone(), &mut errors)
    });
    stack.push(output_matrix.into());
    Ok(CommandOutput::from_errors(errors))
  }

  fn as_subcommand(&self, _opts: &CommandOptions) -> Option<Subcommand> {
    None
  }
}

impl Command for InnerProductCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    context: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    let (mult_id, add_id) = validate_schema(&binary_subcommand_argument_schema(), args)?;
    let calculation_mode = state.calculation_mode().clone();
    let mut errors = ErrorList::new();
    let simplifier = context.simplifier.as_ref();
    state.undo_stack_mut().push_cut();
    let mut stack = KeepableStack::new(state.main_stack_mut(), context.opts.keep_modifier);

    let mult_subcommand = mult_id.as_ref().get_subcommand(context.dispatch_table)?;
    let add_subcommand = add_id.as_ref().get_subcommand(context.dispatch_table)?;
    anyhow::ensure!(mult_subcommand.arity() == 2, "Expected binary subcommand");
    anyhow::ensure!(add_subcommand.arity() == 2, "Expected binary subcommand");

    let prism = prisms::ExprToVector.and(prisms::ExprToVector);

    let [a_vec, b_vec] = stack.pop_several(2)?.try_into().unwrap();
    let (a_vec, b_vec) = match prism.narrow_type((a_vec, b_vec)) {
      Ok(values) => values,
      Err((a_vec, b_vec)) => {
        if !context.opts.keep_modifier {
          stack.push_several([a_vec, b_vec]);
        }
        anyhow::bail!("Expected two vectors");
      }
    };
    if a_vec.len() != b_vec.len() {
      if !context.opts.keep_modifier {
        stack.push_several([prisms::ExprToVector.widen_type(a_vec), prisms::ExprToVector.widen_type(b_vec)]);
      }
      anyhow::bail!("Vector length mismatch");
    }
    if a_vec.is_empty() {
      if !context.opts.keep_modifier {
        stack.push_several([prisms::ExprToVector.widen_type(a_vec), prisms::ExprToVector.widen_type(b_vec)]);
      }
      anyhow::bail!("Expected non-empty vectors");
    }

    // call_or_panic: We checked the arity above.
    //
    // Note: We have to collect the intermediate results into a
    // vector, since we can't double borrow `errors`.
    let intermediate_vec: Vec<_> = a_vec.into_iter().zip(b_vec)
      .map(|(a, b)| mult_subcommand.call_or_panic(vec![a, b], simplifier, calculation_mode.clone(), &mut errors))
      .collect();
    let expr = intermediate_vec.into_iter()
      .reduce(|acc, x| add_subcommand.call_or_panic(vec![acc, x], simplifier, calculation_mode.clone(), &mut errors))
      .unwrap();
    stack.push(expr);
    Ok(CommandOutput::from_errors(errors))
  }

  fn as_subcommand(&self, _opts: &CommandOptions) -> Option<Subcommand> {
    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::stack::test_utils::stack_of;
  use crate::stack::StackError;
  use crate::expr::Expr;
  use crate::command::test_utils::act_on_stack;
  use crate::command::options::CommandOptions;
  use crate::command::subcommand::{SubcommandArityError, GetSubcommandError};
  use crate::command::subcommand::SubcommandId;
  use crate::command::functional::{UnaryFunctionCommand, BinaryFunctionCommand};
  use crate::command::nullary::NullaryCommand;
  use crate::command::dispatch::CommandDispatchTable;

  use once_cell::sync::Lazy;

  use std::collections::HashMap;

  fn sample_dispatch_table() -> CommandDispatchTable {
    let mut hash_map = HashMap::<String, Box<dyn Command + Send + Sync>>::new();
    hash_map.insert("nop".to_string(), Box::new(NullaryCommand));
    hash_map.insert("test_func".to_string(), Box::new(UnaryFunctionCommand::named("test_func")));
    hash_map.insert("test_func2".to_string(), Box::new(BinaryFunctionCommand::named("test_func2")));
    hash_map.insert("+".to_string(), Box::new(BinaryFunctionCommand::named("+")));
    hash_map.insert("*".to_string(), Box::new(BinaryFunctionCommand::named("*")));
    CommandDispatchTable::from_hash_map(hash_map)
  }

  /// Compatible with the
  /// [`ActOnStackArg`](crate::command::test_utils::ActOnStackArg)
  /// interface.
  fn setup_sample_dispatch_table(_args: &mut Vec<String>, context: &mut CommandContext) {
    static TABLE: Lazy<CommandDispatchTable> = Lazy::new(sample_dispatch_table);
    let concrete_table = Lazy::force(&TABLE);
    context.dispatch_table = concrete_table;
  }

  fn subcommand(name: &str) -> String {
    let subcommand_id = SubcommandId { name: String::from(name), options: CommandOptions::default() };
    serde_json::to_string(&subcommand_id).unwrap()
  }

  #[test]
  fn test_apply_command_unary() {
    let command = VectorApplyCommand::new();
    let arg = subcommand("test_func");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![Expr::from(30)]),
    ];
    let output_stack = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("test_func", vec![Expr::from(30)]),
    ]));
  }

  #[test]
  fn test_apply_command_unary_with_keep_modifier() {
    let command = VectorApplyCommand::new();
    let arg = subcommand("test_func");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![Expr::from(30)]),
    ];
    let opts = CommandOptions::default().with_keep_modifier();
    let output_stack = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg], opts), input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![Expr::from(30)]),
      Expr::call("test_func", vec![Expr::from(30)]),
    ]));
  }

  #[test]
  fn test_apply_command_on_empty_stack() {
    let command = VectorApplyCommand::new();
    let arg = subcommand("test_func");
    let input_stack = Vec::<Expr>::new();
    let err = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(err, StackError::NotEnoughElements { expected: 1, actual: 0 });
  }

  #[test]
  fn test_apply_command_type_error() {
    let command = VectorApplyCommand::new();
    let arg = subcommand("test_func");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::from(30),
    ];
    let err = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap_err();
    assert_eq!(err.to_string(), "Expected vector");
  }

  #[test]
  fn test_apply_command_arity_error() {
    let command = VectorApplyCommand::new();
    let arg = subcommand("test_func");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![Expr::from(30), Expr::from(40)]),
    ];
    let err = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap_err();
    let err = err.downcast::<SubcommandArityError>().unwrap();
    assert!(matches!(err, SubcommandArityError { expected: 1, actual: 2, args: _ }));
  }

  #[test]
  fn test_apply_command_binary() {
    let command = VectorApplyCommand::new();
    let arg = subcommand("test_func2");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![Expr::from(30), Expr::from(40)]),
    ];
    let output_stack = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("test_func2", vec![Expr::from(30), Expr::from(40)]),
    ]));
  }

  #[test]
  fn test_apply_command_binary_with_keep_modifier() {
    let command = VectorApplyCommand::new();
    let arg = subcommand("test_func2");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![Expr::from(30), Expr::from(40)]),
    ];
    let opts = CommandOptions::default().with_keep_modifier();
    let output_stack = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg], opts), input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![Expr::from(30), Expr::from(40)]),
      Expr::call("test_func2", vec![Expr::from(30), Expr::from(40)]),
    ]));
  }

  #[test]
  fn test_apply_command_on_nonexistent_subcommand() {
    let command = VectorApplyCommand::new();
    let arg = subcommand("nonexistent");
    let input_stack = vec![
      Expr::call("vector", vec![Expr::from(10)]),
    ];
    let err = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap_err();
    let err = err.downcast::<GetSubcommandError>().unwrap();
    assert!(matches!(err, GetSubcommandError::NoSuchCommandError(_)));
  }

  #[test]
  fn test_apply_command_on_invalid_subcommand() {
    let command = VectorApplyCommand::new();
    let arg = subcommand("nop");
    let input_stack = vec![
      Expr::call("vector", vec![Expr::from(10)]),
    ];
    let err = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap_err();
    let err = err.downcast::<GetSubcommandError>().unwrap();
    assert!(matches!(err, GetSubcommandError::InvalidSubcommandError(_)));
  }

  #[test]
  fn test_map_command() {
    let command = VectorMapCommand::new();
    let arg = subcommand("test_func");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::from(30),
        Expr::from(40),
        Expr::from(50),
      ]),
    ];
    let output_stack = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::call("test_func", vec![Expr::from(30)]),
        Expr::call("test_func", vec![Expr::from(40)]),
        Expr::call("test_func", vec![Expr::from(50)]),
      ]),
    ]));
  }

  #[test]
  fn test_map_command_with_keep_modifier() {
    let command = VectorMapCommand::new();
    let arg = subcommand("test_func");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::from(30),
        Expr::from(40),
        Expr::from(50),
      ]),
    ];
    let opts = CommandOptions::default().with_keep_modifier();
    let output_stack = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg], opts), input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::from(30),
        Expr::from(40),
        Expr::from(50),
      ]),
      Expr::call("vector", vec![
        Expr::call("test_func", vec![Expr::from(30)]),
        Expr::call("test_func", vec![Expr::from(40)]),
        Expr::call("test_func", vec![Expr::from(50)]),
      ]),
    ]));
  }

  #[test]
  fn test_map_command_on_empty_vec() {
    let command = VectorMapCommand::new();
    let arg = subcommand("test_func");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![]),
    ];
    let output_stack = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![]),
    ]));
  }

  #[test]
  fn test_map_command_on_non_vector() {
    let command = VectorMapCommand::new();
    let arg = subcommand("test_func");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::from(30),
    ];
    let err = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap_err();
    assert_eq!(err.to_string(), "Expected vector");

  }

  #[test]
  fn test_map_command_with_subcommand_of_wrong_arity() {
    let command = VectorMapCommand::new();
    let arg = subcommand("test_func2");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![]),
    ];
    let err = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap_err();
    assert_eq!(err.to_string(), "Expected unary subcommand");
  }

  #[test]
  fn test_reduce_command() {
    let command = VectorReduceCommand::new(ReduceDir::LeftToRight);
    let arg = subcommand("test_func2");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::from(30),
        Expr::from(40),
        Expr::from(50),
      ]),
    ];
    let output_stack = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("test_func2", vec![
        Expr::call("test_func2", vec![
          Expr::from(30),
          Expr::from(40),
        ]),
        Expr::from(50),
      ]),
    ]));
  }

  #[test]
  fn test_reduce_command_right_to_left() {
    let command = VectorReduceCommand::new(ReduceDir::RightToLeft);
    let arg = subcommand("test_func2");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::from(30),
        Expr::from(40),
        Expr::from(50),
      ]),
    ];
    let output_stack = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("test_func2", vec![
        Expr::from(30),
        Expr::call("test_func2", vec![
          Expr::from(40),
          Expr::from(50),
        ]),
      ]),
    ]));
  }

  #[test]
  fn test_reduce_command_with_keep_modifier() {
    let command = VectorReduceCommand::new(ReduceDir::LeftToRight);
    let arg = subcommand("test_func2");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::from(30),
        Expr::from(40),
        Expr::from(50),
      ]),
    ];
    let opts = CommandOptions::default().with_keep_modifier();
    let output_stack = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg], opts), input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::from(30),
        Expr::from(40),
        Expr::from(50),
      ]),
      Expr::call("test_func2", vec![
        Expr::call("test_func2", vec![
          Expr::from(30),
          Expr::from(40),
        ]),
        Expr::from(50),
      ]),
    ]));
  }

  #[test]
  fn test_reduce_command_right_to_left_with_keep_modifier() {
    let command = VectorReduceCommand::new(ReduceDir::RightToLeft);
    let arg = subcommand("test_func2");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::from(30),
        Expr::from(40),
        Expr::from(50),
      ]),
    ];
    let opts = CommandOptions::default().with_keep_modifier();
    let output_stack = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg], opts), input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::from(30),
        Expr::from(40),
        Expr::from(50),
      ]),
      Expr::call("test_func2", vec![
        Expr::from(30),
        Expr::call("test_func2", vec![
          Expr::from(40),
          Expr::from(50),
        ]),
      ]),
    ]));
  }

  #[test]
  fn test_reduce_command_type_error() {
    let command = VectorReduceCommand::new(ReduceDir::LeftToRight);
    let arg = subcommand("test_func2");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("some_other_function", vec![
        Expr::from(30),
        Expr::from(40),
        Expr::from(50),
      ]),
    ];
    let err = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap_err();
    assert_eq!(err.to_string(), "Expected vector");
  }

  #[test]
  fn test_reduce_command_empty_vec_error() {
    let command = VectorReduceCommand::new(ReduceDir::LeftToRight);
    let arg = subcommand("test_func2");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![]),
    ];
    let err = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap_err();
    assert_eq!(err.to_string(), "Expected non-empty vector");
  }

  #[test]
  fn test_reduce_command_arity_error() {
    let command = VectorReduceCommand::new(ReduceDir::LeftToRight);
    let arg = subcommand("test_func");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::from(30),
        Expr::from(40),
        Expr::from(50),
      ]),
    ];
    let err = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap_err();
    assert_eq!(err.to_string(), "Expected binary subcommand");
  }

  #[test]
  fn test_reduce_command_on_empty_stack() {
    let command = VectorReduceCommand::new(ReduceDir::LeftToRight);
    let arg = subcommand("test_func2");
    let input_stack = Vec::<Expr>::new();
    let err = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(err, StackError::NotEnoughElements { expected: 1, actual: 0 });
  }

  #[test]
  fn test_reduce_command_on_invalid_subcommand() {
    let command = VectorReduceCommand::new(ReduceDir::LeftToRight);
    let arg = subcommand("nop");
    let input_stack = vec![
      Expr::call("vector", vec![Expr::from(10)]),
    ];
    let err = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap_err();
    let err = err.downcast::<GetSubcommandError>().unwrap();
    assert!(matches!(err, GetSubcommandError::InvalidSubcommandError(_)));
  }

  #[test]
  fn test_reduce_command_on_vector_of_len_one() {
    let command = VectorReduceCommand::new(ReduceDir::LeftToRight);
    let arg = subcommand("test_func2");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::from(30),
      ]),
    ];
    let output_stack = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::from(30),
    ]));
  }

  #[test]
  fn test_reduce_command_right_to_left_on_vector_of_len_one() {
    let command = VectorReduceCommand::new(ReduceDir::RightToLeft);
    let arg = subcommand("test_func2");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::from(30),
      ]),
    ];
    let output_stack = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::from(30),
    ]));
  }

  #[test]
  fn test_accum_command() {
    let command = VectorAccumCommand::new(ReduceDir::LeftToRight);
    let arg = subcommand("test_func2");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::from(30),
        Expr::from(40),
        Expr::from(50),
      ]),
    ];
    let output_stack = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::from(30),
        Expr::call("test_func2", vec![
          Expr::from(30),
          Expr::from(40),
        ]),
        Expr::call("test_func2", vec![
          Expr::call("test_func2", vec![
            Expr::from(30),
            Expr::from(40),
          ]),
          Expr::from(50),
        ]),
      ]),
    ]));
  }

  #[test]
  fn test_accum_command_right_to_left() {
    let command = VectorAccumCommand::new(ReduceDir::RightToLeft);
    let arg = subcommand("test_func2");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::from(30),
        Expr::from(40),
        Expr::from(50),
      ]),
    ];
    let output_stack = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::call("test_func2", vec![
          Expr::from(30),
          Expr::call("test_func2", vec![
            Expr::from(40),
            Expr::from(50),
          ]),
        ]),
        Expr::call("test_func2", vec![
          Expr::from(40),
          Expr::from(50),
        ]),
        Expr::from(50),
      ]),
    ]));
  }

  #[test]
  fn test_accum_command_with_keep_modifier() {
    let command = VectorAccumCommand::new(ReduceDir::LeftToRight);
    let arg = subcommand("test_func2");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::from(30),
        Expr::from(40),
        Expr::from(50),
      ]),
    ];
    let opts = CommandOptions::default().with_keep_modifier();
    let output_stack = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg], opts), input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::from(30),
        Expr::from(40),
        Expr::from(50),
      ]),
      Expr::call("vector", vec![
        Expr::from(30),
        Expr::call("test_func2", vec![
          Expr::from(30),
          Expr::from(40),
        ]),
        Expr::call("test_func2", vec![
          Expr::call("test_func2", vec![
            Expr::from(30),
            Expr::from(40),
          ]),
          Expr::from(50),
        ]),
      ]),
    ]));
  }

  #[test]
  fn test_accum_command_right_to_left_with_keep_modifier() {
    let command = VectorAccumCommand::new(ReduceDir::RightToLeft);
    let arg = subcommand("test_func2");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::from(30),
        Expr::from(40),
        Expr::from(50),
      ]),
    ];
    let opts = CommandOptions::default().with_keep_modifier();
    let output_stack = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg], opts), input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::from(30),
        Expr::from(40),
        Expr::from(50),
      ]),
      Expr::call("vector", vec![
        Expr::call("test_func2", vec![
          Expr::from(30),
          Expr::call("test_func2", vec![
            Expr::from(40),
            Expr::from(50),
          ]),
        ]),
        Expr::call("test_func2", vec![
          Expr::from(40),
          Expr::from(50),
        ]),
        Expr::from(50),
      ]),
    ]));
  }

  #[test]
  fn test_accum_command_type_error() {
    let command = VectorAccumCommand::new(ReduceDir::LeftToRight);
    let arg = subcommand("test_func2");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("some_other_function", vec![
        Expr::from(30),
        Expr::from(40),
        Expr::from(50),
      ]),
    ];
    let err = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap_err();
    assert_eq!(err.to_string(), "Expected vector");
  }

  #[test]
  fn test_accum_command_empty_vec_error() {
    let command = VectorAccumCommand::new(ReduceDir::LeftToRight);
    let arg = subcommand("test_func2");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![]),
    ];
    let output_stack = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![]),
    ]));
  }

  #[test]
  fn test_accum_command_arity_error() {
    let command = VectorAccumCommand::new(ReduceDir::LeftToRight);
    let arg = subcommand("test_func");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::from(30),
        Expr::from(40),
        Expr::from(50),
      ]),
    ];
    let err = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap_err();
    assert_eq!(err.to_string(), "Expected binary subcommand");
  }

  #[test]
  fn test_accum_command_on_empty_stack() {
    let command = VectorAccumCommand::new(ReduceDir::LeftToRight);
    let arg = subcommand("test_func2");
    let input_stack = Vec::<Expr>::new();
    let err = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(err, StackError::NotEnoughElements { expected: 1, actual: 0 });
  }

  #[test]
  fn test_accum_command_on_invalid_subcommand() {
    let command = VectorAccumCommand::new(ReduceDir::LeftToRight);
    let arg = subcommand("nop");
    let input_stack = vec![
      Expr::call("vector", vec![Expr::from(10)]),
    ];
    let err = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap_err();
    let err = err.downcast::<GetSubcommandError>().unwrap();
    assert!(matches!(err, GetSubcommandError::InvalidSubcommandError(_)));
  }

  #[test]
  fn test_accum_command_on_vector_of_len_one() {
    let command = VectorAccumCommand::new(ReduceDir::LeftToRight);
    let arg = subcommand("test_func2");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::from(30),
      ]),
    ];
    let output_stack = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::from(30),
      ]),
    ]));
  }

  #[test]
  fn test_accum_command_right_to_left_on_vector_of_len_one() {
    let command = VectorAccumCommand::new(ReduceDir::RightToLeft);
    let arg = subcommand("test_func2");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::from(30),
      ]),
    ];
    let output_stack = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::from(30),
      ]),
    ]));
  }

  #[test]
  fn test_outer_product() {
    let command = OuterProductCommand::new();
    let arg = subcommand("test_func2");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::from(30),
        Expr::from(40),
      ]),
      Expr::call("vector", vec![
        Expr::from(50),
        Expr::from(60),
        Expr::from(70),
      ]),
    ];
    let output_stack = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::call("vector", vec![
          Expr::call("test_func2", vec![Expr::from(30), Expr::from(50)]),
          Expr::call("test_func2", vec![Expr::from(30), Expr::from(60)]),
          Expr::call("test_func2", vec![Expr::from(30), Expr::from(70)]),
        ]),
        Expr::call("vector", vec![
          Expr::call("test_func2", vec![Expr::from(40), Expr::from(50)]),
          Expr::call("test_func2", vec![Expr::from(40), Expr::from(60)]),
          Expr::call("test_func2", vec![Expr::from(40), Expr::from(70)]),
        ]),
      ]),
    ]));
  }

  #[test]
  fn test_outer_product_with_keep_modifier() {
    let command = OuterProductCommand::new();
    let arg = subcommand("test_func2");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::from(30),
        Expr::from(40),
      ]),
      Expr::call("vector", vec![
        Expr::from(50),
        Expr::from(60),
        Expr::from(70),
      ]),
    ];
    let opts = CommandOptions::default().with_keep_modifier();
    let output_stack = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg], opts), input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::from(30),
        Expr::from(40),
      ]),
      Expr::call("vector", vec![
        Expr::from(50),
        Expr::from(60),
        Expr::from(70),
      ]),
      Expr::call("vector", vec![
        Expr::call("vector", vec![
          Expr::call("test_func2", vec![Expr::from(30), Expr::from(50)]),
          Expr::call("test_func2", vec![Expr::from(30), Expr::from(60)]),
          Expr::call("test_func2", vec![Expr::from(30), Expr::from(70)]),
        ]),
        Expr::call("vector", vec![
          Expr::call("test_func2", vec![Expr::from(40), Expr::from(50)]),
          Expr::call("test_func2", vec![Expr::from(40), Expr::from(60)]),
          Expr::call("test_func2", vec![Expr::from(40), Expr::from(70)]),
        ]),
      ]),
    ]));
  }

  #[test]
  fn test_inner_product() {
    let command = InnerProductCommand::new();
    let mul_arg = subcommand("*");
    let add_arg = subcommand("+");
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("vector", vec![
        Expr::from(30),
        Expr::from(40),
        Expr::from(50),
      ]),
      Expr::call("vector", vec![
        Expr::from(60),
        Expr::from(70),
        Expr::from(80),
      ]),
    ];
    let output_stack = act_on_stack(&command, (setup_sample_dispatch_table, vec![mul_arg, add_arg]), input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("+", vec![
        Expr::call("+", vec![
          Expr::call("*", vec![Expr::from(30), Expr::from(60)]),
          Expr::call("*", vec![Expr::from(40), Expr::from(70)]),
        ]),
        Expr::call("*", vec![Expr::from(50), Expr::from(80)]),
      ]),
    ]));
  }
}
