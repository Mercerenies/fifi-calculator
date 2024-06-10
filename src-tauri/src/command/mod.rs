
pub mod arguments;
mod base;
pub mod dispatch;
pub mod functional;
pub mod general;
pub mod options;
pub mod shuffle;
pub mod input;
pub mod variables;

pub use base::{Command, CommandContext, CommandOutput};
use functional::{UnaryFunctionCommand, BinaryFunctionCommand};
use dispatch::CommandDispatchTable;
use input::{push_number_command, push_expr_command};
use crate::expr::Expr;
use crate::expr::number::ComplexNumber;
use crate::state::ApplicationState;

use std::collections::HashMap;

pub fn default_dispatch_table() -> CommandDispatchTable {
  let mut map: HashMap<String, Box<dyn Command + Send + Sync>> = HashMap::new();

  // Nullary commands
  map.insert("+".to_string(), Box::new(BinaryFunctionCommand::named("+")));
  map.insert("-".to_string(), Box::new(BinaryFunctionCommand::named("-")));
  map.insert("*".to_string(), Box::new(BinaryFunctionCommand::named("*")));
  map.insert("/".to_string(), Box::new(BinaryFunctionCommand::named("/")));
  map.insert("%".to_string(), Box::new(BinaryFunctionCommand::named("%")));
  map.insert("div".to_string(), Box::new(BinaryFunctionCommand::named("div")));
  map.insert("^".to_string(), Box::new(BinaryFunctionCommand::named("^")));
  map.insert("ln".to_string(), Box::new(UnaryFunctionCommand::named("ln")));
  map.insert("log".to_string(), Box::new(BinaryFunctionCommand::named("log")));
  map.insert("log10".to_string(), Box::new(UnaryFunctionCommand::new(log10)));
  map.insert("log2".to_string(), Box::new(UnaryFunctionCommand::new(log2)));
  map.insert("*i".to_string(), Box::new(UnaryFunctionCommand::new(times_i)));
  map.insert("negate".to_string(), Box::new(UnaryFunctionCommand::new(times_minus_one)));
  map.insert("pop".to_string(), Box::new(shuffle::PopCommand));
  map.insert("swap".to_string(), Box::new(shuffle::SwapCommand));
  map.insert("dup".to_string(), Box::new(shuffle::DupCommand));
  map.insert("substitute_vars".to_string(), Box::new(UnaryFunctionCommand::with_state(substitute_vars)));

  // Commands which accept a single string.
  map.insert("push_number".to_string(), Box::new(push_number_command()));
  map.insert("push_expr".to_string(), Box::new(push_expr_command()));

  // Variable-related commands
  map.insert("manual_substitute".to_string(), Box::new(variables::SubstituteVarCommand::new())); // TODO: Reify as function?
  map.insert("store_var".to_string(), Box::new(variables::StoreVarCommand::new()));

  CommandDispatchTable::from_hash_map(map)
}

fn log10(expr: Expr) -> Expr {
  Expr::call("log", vec![expr, Expr::from(10)])
}

fn log2(expr: Expr) -> Expr {
  Expr::call("log", vec![expr, Expr::from(2)])
}

fn times_i(expr: Expr) -> Expr {
  let ii = ComplexNumber::ii();
  Expr::call("*", vec![expr, Expr::from(ii)])
}

fn times_minus_one(expr: Expr) -> Expr {
  Expr::call("*", vec![expr, Expr::from(-1)])
}

fn substitute_vars(expr: Expr, state: &ApplicationState) -> Expr {
  let var_table = state.variable_table();
  expr.substitute_vars(var_table)
}

#[cfg(test)]
pub(crate) mod test_utils {
  use super::*;
  use crate::expr::Expr;
  use crate::command::options::CommandOptions;
  use crate::state::test_utils::state_for_stack;
  use crate::stack::test_utils::stack_of;
  use crate::stack::{Stack, StackError};

  /// Tests the operation on the given input stack, expecting a
  /// success. Passes no string arguments.
  pub fn act_on_stack(command: &impl Command, opts: CommandOptions, input_stack: Vec<i64>) -> Stack<Expr> {
    let args = Vec::<String>::new();
    act_on_stack_with_args(command, args, opts, input_stack)
  }

  /// Tests the operation on the given input stack. Expects a failure.
  /// Passes no string arguments. Asserts that the stack is unchanged
  /// and returns the error.
  pub fn act_on_stack_err(command: &impl Command, opts: CommandOptions, input_stack: Vec<i64>) -> StackError {
    let args = Vec::<String>::new();
    act_on_stack_with_args_err(command, args, opts, input_stack)
  }

  /// Tests the operation on the given input stack, expecting a
  /// success.
  pub fn act_on_stack_with_args(
    command: &impl Command,
    args: Vec<impl Into<String>>,
    opts: CommandOptions,
    input_stack: Vec<i64>,
  ) -> Stack<Expr> {
    let args = args.into_iter().map(|s| s.into()).collect();
    let mut state = state_for_stack(input_stack);
    let mut context = CommandContext::default();
    context.opts = opts;
    let output = command.run_command(&mut state, args, &context).unwrap();
    assert!(output.errors.is_empty());
    state.into_main_stack()
  }

  /// Tests the operation on the given input stack. Expects a failure.
  /// Asserts that the stack is unchanged and returns the error.
  pub fn act_on_stack_with_args_err(
    command: &impl Command,
    args: Vec<impl Into<String>>,
    opts: CommandOptions,
    input_stack: Vec<i64>,
  ) -> StackError {
    let args = args.into_iter().map(|s| s.into()).collect();
    let mut state = state_for_stack(input_stack.clone());
    let mut context = CommandContext::default();
    context.opts = opts;
    let err = command.run_command(&mut state, args, &context).unwrap_err();
    let err = match err.downcast::<StackError>() {
      Ok(stack_error) => stack_error,
      Err(other_error) => { panic!("Expected StackError, got {:?}", other_error); }
    };
    assert_eq!(state.into_main_stack(), stack_of(input_stack));
    err
  }
}
