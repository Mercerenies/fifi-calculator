
pub mod arguments;
mod base;
pub mod dispatch;
pub mod functional;
pub mod general;
pub mod options;
pub mod shuffle;
pub mod input;

pub use base::{Command, CommandContext, CommandOutput};
use functional::{UnaryFunctionCommand, BinaryFunctionCommand};
use dispatch::CommandDispatchTable;
use crate::expr::Expr;
use crate::expr::number::ComplexNumber;
use crate::state::ApplicationState;

use std::collections::HashMap;

pub fn default_dispatch_table() -> CommandDispatchTable {
  let mut map: HashMap<String, Box<dyn Command + Send + Sync>> = HashMap::new();
  map.insert("+".to_string(), Box::new(BinaryFunctionCommand::named("+")));
  map.insert("-".to_string(), Box::new(BinaryFunctionCommand::named("-")));
  map.insert("*".to_string(), Box::new(BinaryFunctionCommand::named("*")));
  map.insert("/".to_string(), Box::new(BinaryFunctionCommand::named("/")));
  map.insert("%".to_string(), Box::new(BinaryFunctionCommand::named("%")));
  map.insert("div".to_string(), Box::new(BinaryFunctionCommand::named("div")));
  map.insert("^".to_string(), Box::new(BinaryFunctionCommand::named("^")));
  map.insert("*i".to_string(), Box::new(UnaryFunctionCommand::new(times_i)));
  map.insert("negate".to_string(), Box::new(UnaryFunctionCommand::new(times_minus_one)));
  map.insert("pop".to_string(), Box::new(shuffle::PopCommand));
  map.insert("swap".to_string(), Box::new(shuffle::SwapCommand));
  map.insert("dup".to_string(), Box::new(shuffle::DupCommand));
  map.insert("substitute_vars".to_string(), Box::new(UnaryFunctionCommand::with_state(substitute_vars)));
  CommandDispatchTable::from_hash_map(map)
}

fn times_i(expr: Expr) -> Expr {
  let ii = ComplexNumber::ii();
  Expr::Call("*".to_string(), vec![expr, Expr::from(ii)])
}

fn times_minus_one(expr: Expr) -> Expr {
  Expr::Call("*".to_string(), vec![expr, Expr::from(-1)])
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
  use crate::error::Error;

  /// Tests the operation on the given input stack, expecting a
  /// success.
  pub fn act_on_stack(command: &impl Command, opts: CommandOptions, input_stack: Vec<i64>) -> Stack<Expr> {
    let mut state = state_for_stack(input_stack);
    let mut context = CommandContext::default();
    context.opts = opts;
    let output = command.run_command(&mut state, vec![], &context).unwrap();
    assert!(output.errors.is_empty());
    state.into_main_stack()
  }

  /// Tests the operation on the given input stack. Expects a failure.
  /// Asserts that the stack is unchanged and returns the error.
  pub fn act_on_stack_err(command: &impl Command, opts: CommandOptions, input_stack: Vec<i64>) -> StackError {
    let mut state = state_for_stack(input_stack.clone());
    let mut context = CommandContext::default();
    context.opts = opts;
    let err = command.run_command(&mut state, vec![], &context).unwrap_err();
    let Error::StackError(err) = err else {
      panic!("Expected StackError, got {:?}", err)
    };
    assert_eq!(state.into_main_stack(), stack_of(input_stack));
    err
  }
}
