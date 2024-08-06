
//! Commands which reduce or operate over vectors, usually using a
//! subcommand in a higher-order capacity.
//!
//! For first-order vector commands, see the [`vector`
//! module](crate::command::vector).

use super::arguments::{UnaryArgumentSchema, validate_schema};
use super::base::{Command, CommandContext, CommandOutput};
use super::options::CommandOptions;
use super::subcommand::{Subcommand, StringToSubcommandId, ParsedSubcommandId};
use crate::util;
use crate::util::prism::Prism;
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
#[derive(Debug)]
pub struct VectorReduceCommand {
  direction: ReduceDir,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReduceDir {
  LeftToRight,
  RightToLeft,
}

fn unary_subcommand_argument_schema() -> UnaryArgumentSchema<StringToSubcommandId, ParsedSubcommandId> {
  UnaryArgumentSchema::new(
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
    stack.push(expr.into());
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

  #[test]
  fn test_apply_command_unary() {
    let command = VectorApplyCommand::new();
    let arg = {
      let subcommand_id = SubcommandId { name: String::from("test_func"), options: CommandOptions::default() };
      serde_json::to_string(&subcommand_id).unwrap()
    };
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
    let arg = {
      let subcommand_id = SubcommandId { name: String::from("test_func"), options: CommandOptions::default() };
      serde_json::to_string(&subcommand_id).unwrap()
    };
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
    let arg = {
      let subcommand_id = SubcommandId { name: String::from("test_func"), options: CommandOptions::default() };
      serde_json::to_string(&subcommand_id).unwrap()
    };
    let input_stack = Vec::<Expr>::new();
    let err = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(err, StackError::NotEnoughElements { expected: 1, actual: 0 });
  }

  #[test]
  fn test_apply_command_type_error() {
    let command = VectorApplyCommand::new();
    let arg = {
      let subcommand_id = SubcommandId { name: String::from("test_func"), options: CommandOptions::default() };
      serde_json::to_string(&subcommand_id).unwrap()
    };
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
    let arg = {
      let subcommand_id = SubcommandId { name: String::from("test_func"), options: CommandOptions::default() };
      serde_json::to_string(&subcommand_id).unwrap()
    };
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
    let arg = {
      let subcommand_id = SubcommandId { name: String::from("test_func2"), options: CommandOptions::default() };
      serde_json::to_string(&subcommand_id).unwrap()
    };
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
    let arg = {
      let subcommand_id = SubcommandId { name: String::from("test_func2"), options: CommandOptions::default() };
      serde_json::to_string(&subcommand_id).unwrap()
    };
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
    let arg = {
      let subcommand_id = SubcommandId { name: String::from("nonexistent"), options: CommandOptions::default() };
      serde_json::to_string(&subcommand_id).unwrap()
    };
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
    let arg = {
      let subcommand_id = SubcommandId { name: String::from("nop"), options: CommandOptions::default() };
      serde_json::to_string(&subcommand_id).unwrap()
    };
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
    let arg = {
      let subcommand_id = SubcommandId { name: String::from("test_func"), options: CommandOptions::default() };
      serde_json::to_string(&subcommand_id).unwrap()
    };
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
    let arg = {
      let subcommand_id = SubcommandId { name: String::from("test_func"), options: CommandOptions::default() };
      serde_json::to_string(&subcommand_id).unwrap()
    };
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
    let arg = {
      let subcommand_id = SubcommandId { name: String::from("test_func"), options: CommandOptions::default() };
      serde_json::to_string(&subcommand_id).unwrap()
    };
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
    let arg = {
      let subcommand_id = SubcommandId { name: String::from("test_func"), options: CommandOptions::default() };
      serde_json::to_string(&subcommand_id).unwrap()
    };
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
    let arg = {
      let subcommand_id = SubcommandId { name: String::from("test_func2"), options: CommandOptions::default() };
      serde_json::to_string(&subcommand_id).unwrap()
    };
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
    let arg = {
      let subcommand_id = SubcommandId { name: String::from("test_func2"), options: CommandOptions::default() };
      serde_json::to_string(&subcommand_id).unwrap()
    };
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
    let arg = {
      let subcommand_id = SubcommandId { name: String::from("test_func2"), options: CommandOptions::default() };
      serde_json::to_string(&subcommand_id).unwrap()
    };
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
    let arg = {
      let subcommand_id = SubcommandId { name: String::from("test_func2"), options: CommandOptions::default() };
      serde_json::to_string(&subcommand_id).unwrap()
    };
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
    let arg = {
      let subcommand_id = SubcommandId { name: String::from("test_func2"), options: CommandOptions::default() };
      serde_json::to_string(&subcommand_id).unwrap()
    };
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
    let arg = {
      let subcommand_id = SubcommandId { name: String::from("test_func2"), options: CommandOptions::default() };
      serde_json::to_string(&subcommand_id).unwrap()
    };
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
    let arg = {
      let subcommand_id = SubcommandId { name: String::from("test_func2"), options: CommandOptions::default() };
      serde_json::to_string(&subcommand_id).unwrap()
    };
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
    let arg = {
      let subcommand_id = SubcommandId { name: String::from("test_func"), options: CommandOptions::default() };
      serde_json::to_string(&subcommand_id).unwrap()
    };
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
    let arg = {
      let subcommand_id = SubcommandId { name: String::from("test_func2"), options: CommandOptions::default() };
      serde_json::to_string(&subcommand_id).unwrap()
    };
    let input_stack = Vec::<Expr>::new();
    let err = act_on_stack(&command, (setup_sample_dispatch_table, vec![arg]), input_stack).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(err, StackError::NotEnoughElements { expected: 1, actual: 0 });
  }

  #[test]
  fn test_reduce_command_on_invalid_subcommand() {
    let command = VectorReduceCommand::new(ReduceDir::LeftToRight);
    let arg = {
      let subcommand_id = SubcommandId { name: String::from("nop"), options: CommandOptions::default() };
      serde_json::to_string(&subcommand_id).unwrap()
    };
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
    let arg = {
      let subcommand_id = SubcommandId { name: String::from("test_func2"), options: CommandOptions::default() };
      serde_json::to_string(&subcommand_id).unwrap()
    };
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
    let arg = {
      let subcommand_id = SubcommandId { name: String::from("test_func2"), options: CommandOptions::default() };
      serde_json::to_string(&subcommand_id).unwrap()
    };
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
}
