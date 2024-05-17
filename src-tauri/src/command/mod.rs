
mod base;
pub mod dispatch;
pub mod functional;
pub mod general;
pub mod options;
pub mod shuffle;

pub use base::{Command, CommandContext};
use functional::BinaryFunctionCommand;
use dispatch::CommandDispatchTable;

use std::collections::HashMap;

pub fn default_dispatch_table() -> CommandDispatchTable {
  let mut map: HashMap<String, Box<dyn Command + Send + Sync>> = HashMap::new();
  map.insert("+".to_string(), Box::new(BinaryFunctionCommand::named("+")));
  map.insert("-".to_string(), Box::new(BinaryFunctionCommand::named("-")));
  map.insert("*".to_string(), Box::new(BinaryFunctionCommand::named("*")));
  map.insert("/".to_string(), Box::new(BinaryFunctionCommand::named("/")));
  map.insert("%".to_string(), Box::new(BinaryFunctionCommand::named("%")));
  map.insert("\\".to_string(), Box::new(BinaryFunctionCommand::named("\\")));
  map.insert("pop".to_string(), Box::new(shuffle::PopCommand));
  map.insert("swap".to_string(), Box::new(shuffle::SwapCommand));
  map.insert("dup".to_string(), Box::new(shuffle::DupCommand));
  CommandDispatchTable::from_hash_map(map)
}

#[cfg(test)]
pub(crate) mod test_utils {
  use super::*;
  use crate::expr::Expr;
  use crate::state::test_utils::state_for_stack;
  use crate::stack::test_utils::stack_of;
  use crate::stack::Stack;
  use crate::stack::error::StackError;
  use crate::error::Error;

  /// Tests the operation on the given input stack, expecting a
  /// success.
  pub fn act_on_stack(command: &impl Command, arg: Option<i64>, input_stack: Vec<i64>) -> Stack<Expr> {
    let mut state = state_for_stack(input_stack);
    let mut context = CommandContext::default();
    context.opts.argument = arg;
    let output = command.run_command(&mut state, &context).unwrap();
    assert!(output.errors.is_empty());
    state.main_stack
  }

  /// Tests the operation on the given input stack. Expects a failure.
  /// Asserts that the stack is unchanged and returns the error.
  pub fn act_on_stack_err(command: &impl Command, arg: Option<i64>, input_stack: Vec<i64>) -> StackError {
    let mut state = state_for_stack(input_stack.clone());
    let mut context = CommandContext::default();
    context.opts.argument = arg;
    let err = command.run_command(&mut state, &context).unwrap_err();
    let Error::StackError(err) = err else {
      panic!("Expected StackError, got {:?}", err)
    };
    assert_eq!(state.main_stack, stack_of(input_stack));
    err
  }
}
