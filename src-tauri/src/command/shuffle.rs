
//! Commands for shuffling the stack.

use super::base::{Command, CommandContext, CommandOutput};
use crate::state::ApplicationState;
use crate::error::Error;
use crate::stack::shuffle;

/// Pops and discards a single value.
#[derive(Debug, Clone)]
pub struct PopCommand;

/// Swaps the top two stack values.
#[derive(Debug, Clone)]
pub struct SwapCommand;

/// Duplicates the top stack value.
#[derive(Debug, Clone)]
pub struct DupCommand;

impl Command for PopCommand {
  fn run_command(&self, state: &mut ApplicationState, ctx: &CommandContext) -> Result<CommandOutput, Error> {
    let arg = ctx.opts.argument.unwrap_or(1);
    if arg > 0 {
      // Pop N elements
      let _ = shuffle::pop_several(&mut state.main_stack, arg as usize)?;
    } else if arg < 0 {
      // Pop a single specific element
      let _ = shuffle::pop_nth(&mut state.main_stack, (- arg - 1) as usize)?;
    } else {
      // Pop all elements
      state.main_stack.pop_all();
    }
    Ok(CommandOutput::success())
  }
}

impl Command for SwapCommand {
  fn run_command(&self, state: &mut ApplicationState, ctx: &CommandContext) -> Result<CommandOutput, Error> {
    // TODO Use context
    let (a, b) = shuffle::pop_two(&mut state.main_stack)?;
    state.main_stack.push(b);
    state.main_stack.push(a);
    Ok(CommandOutput::success())
  }
}

impl Command for DupCommand {
  fn run_command(&self, state: &mut ApplicationState, ctx: &CommandContext) -> Result<CommandOutput, Error> {
    // TODO Use context
    let a = shuffle::pop_one(&mut state.main_stack)?;
    state.main_stack.push(a.clone());
    state.main_stack.push(a);
    Ok(CommandOutput::success())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::expr::Expr;
  use crate::expr::number::Number;
  use crate::stack::Stack;
  use crate::stack::error::StackError;

  // TODO Dup and swap tests

  fn stack_of(number_vec: Vec<i64>) -> Stack<Expr> {
    let expr_vec: Vec<_> = number_vec.into_iter().map(|n| {
      Expr::from(Number::from(n))
    }).collect();
    Stack::from(expr_vec)
  }

  /// Takes a stack with the top on the right.
  fn example_state(stack_vec: Vec<i64>) -> ApplicationState {
    let mut state = ApplicationState::new();
    state.main_stack = stack_of(stack_vec);
    state
  }

  #[test]
  fn test_simple_pop() {
    let mut state = example_state(vec![10, 20, 30]);
    let output = PopCommand.run_command(&mut state, &CommandContext::default()).unwrap();
    assert!(output.errors.is_empty());
    assert_eq!(state.main_stack, stack_of(vec![10, 20]));
  }

  #[test]
  fn test_simple_pop_on_empty_stack() {
    let mut state = example_state(vec![]);
    let err = PopCommand.run_command(&mut state, &CommandContext::default()).unwrap_err();
    let Error::StackError(err) = err else { panic!("Expected StackError, got {:?}", err) };
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 1, actual: 0 },
    )
  }

  // TODO (/////) other 'pop' cases
}
