
//! Commands for shuffling the stack.

use super::base::{Command, CommandContext, CommandOutput};
use crate::state::ApplicationState;
use crate::stack::keepable::KeepableStack;
use crate::stack::base::{StackLike, RandomAccessStackLike};
use crate::error::Error;

use std::cmp::Ordering;

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
    // Note: PopCommand explicitly ignores the keep_modifier, as it
    // would always be a no-op.
    state.undo_stack_mut().push_cut();
    let mut stack = state.main_stack_mut();

    let arg = ctx.opts.argument.unwrap_or(1);
    match arg.cmp(&0) {
      Ordering::Greater => {
        // Pop N elements
        let _ = stack.pop_several(arg as usize)?;
      }
      Ordering::Less => {
        // Pop a single specific element
        let _ = stack.pop_nth((- arg - 1) as usize)?;
      }
      Ordering::Equal => {
        // Pop all elements
        stack.pop_all();
      }
    }
    Ok(CommandOutput::success())
  }
}

impl Command for SwapCommand {
  fn run_command(&self, state: &mut ApplicationState, ctx: &CommandContext) -> Result<CommandOutput, Error> {
    state.undo_stack_mut().push_cut();
    let mut stack = state.main_stack_mut();
    let mut stack = KeepableStack::new(&mut stack, ctx.opts.keep_modifier);

    let arg = ctx.opts.argument.unwrap_or(2);
    match arg.cmp(&0) {
      Ordering::Greater => {
        // Bury top element N deep.
        let mut elements = stack.pop_several(arg as usize)?;
        stack.push(elements.pop().unwrap()); // unwrap: arg > 0 so elements is non-empty.
        stack.push_several(elements);
      }
      Ordering::Less => {
        // Bury top N elements at bottom.
        let elements_to_bury = stack.pop_several((- arg) as usize)?;
        let rest_of_elements = stack.pop_all();
        stack.push_several(elements_to_bury);
        stack.push_several(rest_of_elements);
      }
      Ordering::Equal => {
        // Reverse stack.
        let mut elements = stack.pop_all();
        elements.reverse();
        stack.push_several(elements);
      }
    }
    Ok(CommandOutput::success())
  }
}

impl Command for DupCommand {
  fn run_command(&self, state: &mut ApplicationState, ctx: &CommandContext) -> Result<CommandOutput, Error> {
    // Note: DupCommand explicitly ignores the keep_modifier, as its
    // behavior would be quite unintuitive (especially with negative
    // numerical arg).
    state.undo_stack_mut().push_cut();
    let mut stack = state.main_stack_mut();

    let arg = ctx.opts.argument.unwrap_or(1);
    match arg.cmp(&0) {
      Ordering::Greater => {
        // Duplicate top N arguments.
        let elements = stack.pop_several(arg as usize)?;
        stack.push_several(elements.clone());
        stack.push_several(elements);
      }
      Ordering::Less => {
        // Duplicate specific element N down.
        let element = stack.get(- arg - 1)?.clone();
        stack.push(element);
      }
      Ordering::Equal => {
        // Duplicate entire stack.
        let elements = stack.pop_all();
        stack.push_several(elements.clone());
        stack.push_several(elements);
      }
    }
    Ok(CommandOutput::success())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::command::test_utils::{act_on_stack, act_on_stack_err};
  use crate::stack::test_utils::stack_of;
  use crate::stack::StackError;

  #[test]
  fn test_simple_pop() {
    let output_stack = act_on_stack(&PopCommand, None, vec![10, 20, 30]);
    assert_eq!(output_stack, stack_of(vec![10, 20]));
  }

  #[test]
  fn test_simple_pop_on_empty_stack() {
    let err = act_on_stack_err(&PopCommand, None, vec![]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 1, actual: 0 },
    )
  }

  #[test]
  fn test_multiple_pop() {
    let output_stack = act_on_stack(&PopCommand, Some(3), vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![10, 20]));
  }

  #[test]
  fn test_multiple_pop_all_stack_elements() {
    let output_stack = act_on_stack(&PopCommand, Some(4), vec![10, 20, 30, 40]);
    assert_eq!(output_stack, stack_of(vec![]));
  }

  #[test]
  fn test_multiple_pop_on_empty_stack() {
    let err = act_on_stack_err(&PopCommand, Some(3), vec![]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 3, actual: 0 },
    );
  }

  #[test]
  fn test_multiple_pop_on_stack_thats_too_small() {
    let err = act_on_stack_err(&PopCommand, Some(4), vec![10, 20, 30]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 4, actual: 3 },
    );
  }

  #[test]
  fn test_pop_with_argument_zero() {
    let output_stack = act_on_stack(&PopCommand, Some(0), vec![10, 20, 30]);
    assert_eq!(output_stack, stack_of(vec![]));
  }

  #[test]
  fn test_pop_with_argument_zero_on_empty_stack() {
    let output_stack = act_on_stack(&PopCommand, Some(0), vec![]);
    assert_eq!(output_stack, stack_of(vec![]));
  }

  #[test]
  fn test_pop_with_negative_one_argument() {
    let output_stack = act_on_stack(&PopCommand, Some(-1), vec![10, 20, 30, 40]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30]));
  }

  #[test]
  fn test_pop_with_negative_one_argument_and_empty_stack() {
    let err = act_on_stack_err(&PopCommand, Some(-1), vec![]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 1, actual: 0 },
    );
  }

  #[test]
  fn test_pop_with_negative_argument() {
    let output_stack = act_on_stack(&PopCommand, Some(-3), vec![10, 20, 30, 40]);
    assert_eq!(output_stack, stack_of(vec![10, 30, 40]));
  }

  #[test]
  fn test_pop_with_negative_argument_and_empty_stack() {
    let err = act_on_stack_err(&PopCommand, Some(-3), vec![]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 3, actual: 0 },
    );
  }

  #[test]
  fn test_pop_with_negative_argument_and_too_small_stack() {
    let err = act_on_stack_err(&PopCommand, Some(-3), vec![10, 20]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 3, actual: 2 },
    );
  }

  #[test]
  fn test_pop_with_negative_argument_at_bottom_of_stack() {
    let output_stack = act_on_stack(&PopCommand, Some(-4), vec![10, 20, 30, 40]);
    assert_eq!(output_stack, stack_of(vec![20, 30, 40]));
  }

  #[test]
  fn test_swap() {
    let output_stack = act_on_stack(&SwapCommand, None, vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 50, 40]));
  }

  #[test]
  fn test_swap_on_stack_size_two() {
    let output_stack = act_on_stack(&SwapCommand, None, vec![10, 20]);
    assert_eq!(output_stack, stack_of(vec![20, 10]));
  }

  #[test]
  fn test_swap_on_stack_size_one() {
    let err = act_on_stack_err(&SwapCommand, None, vec![10]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 2, actual: 1 },
    );
  }

  #[test]
  fn test_swap_on_empty_stack() {
    let err = act_on_stack_err(&SwapCommand, None, vec![]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 2, actual: 0 },
    );
  }

  #[test]
  fn test_swap_positive_arg() {
    let output_stack = act_on_stack(&SwapCommand, Some(4), vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![10, 50, 20, 30, 40]));
  }

  #[test]
  fn test_swap_positive_arg_equal_to_stack_size() {
    let output_stack = act_on_stack(&SwapCommand, Some(3), vec![10, 20, 30]);
    assert_eq!(output_stack, stack_of(vec![30, 10, 20]));
  }

  #[test]
  fn test_swap_with_positive_arg_and_too_small_stack() {
    let err = act_on_stack_err(&SwapCommand, Some(3), vec![10, 20]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 3, actual: 2 },
    );
  }

  #[test]
  fn test_swap_with_positive_arg_on_empty_stack() {
    let err = act_on_stack_err(&SwapCommand, Some(4), vec![]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 4, actual: 0 },
    );
  }

  #[test]
  fn test_swap_arg_of_one() {
    let output_stack = act_on_stack(&SwapCommand, Some(1), vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50]));
  }

  #[test]
  fn test_swap_arg_of_one_on_empty_stack() {
    let err = act_on_stack_err(&SwapCommand, Some(1), vec![]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 1, actual: 0 },
    );
  }

  #[test]
  fn test_swap_argument_zero() {
    let output_stack = act_on_stack(&SwapCommand, Some(0), vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![50, 40, 30, 20, 10]));
  }

  #[test]
  fn test_swap_argument_zero_on_stack_size_one() {
    let output_stack = act_on_stack(&SwapCommand, Some(0), vec![10]);
    assert_eq!(output_stack, stack_of(vec![10]));
  }

  #[test]
  fn test_swap_argument_zero_on_empty_stack() {
    let output_stack = act_on_stack(&SwapCommand, Some(0), vec![]);
    assert_eq!(output_stack, stack_of(vec![]));
  }

  #[test]
  fn test_swap_with_negative_one_arg() {
    let output_stack = act_on_stack(&SwapCommand, Some(-1), vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![50, 10, 20, 30, 40]));
  }

  #[test]
  fn test_swap_with_negative_one_arg_on_stack_size_one() {
    let output_stack = act_on_stack(&SwapCommand, Some(-1), vec![10]);
    assert_eq!(output_stack, stack_of(vec![10]));
  }

  #[test]
  fn test_swap_with_negative_one_arg_on_empty_stack() {
    let err = act_on_stack_err(&SwapCommand, Some(-1), vec![]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 1, actual: 0 },
    );
  }

  #[test]
  fn test_swap_with_negative_arg() {
    let output_stack = act_on_stack(&SwapCommand, Some(-3), vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![30, 40, 50, 10, 20]));
  }

  #[test]
  fn test_swap_with_negative_arg_whole_stack() {
    let output_stack = act_on_stack(&SwapCommand, Some(-5), vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50]));
  }

  #[test]
  fn test_swap_with_negative_arg_and_too_small_stack() {
    let err = act_on_stack_err(&SwapCommand, Some(-3), vec![10, 20]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 3, actual: 2 },
    )
  }

  #[test]
  fn test_swap_with_negative_arg_and_empty_stack() {
    let err = act_on_stack_err(&SwapCommand, Some(-3), vec![]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 3, actual: 0 },
    )
  }

  #[test]
  fn test_dup() {
    let output_stack = act_on_stack(&DupCommand, None, vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 50]));
  }

  #[test]
  fn test_dup_on_stack_size_one() {
    let output_stack = act_on_stack(&DupCommand, None, vec![10]);
    assert_eq!(output_stack, stack_of(vec![10, 10]));
  }

  #[test]
  fn test_dup_on_empty_stack() {
    let err = act_on_stack_err(&DupCommand, None, vec![]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 1, actual: 0 },
    );
  }

  #[test]
  fn test_dup_positive_arg() {
    let output_stack = act_on_stack(&DupCommand, Some(2), vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 40, 50]));
  }

  #[test]
  fn test_dup_positive_arg_equal_to_stack_size() {
    let output_stack = act_on_stack(&DupCommand, Some(3), vec![10, 20, 30]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 10, 20, 30]));
  }

  #[test]
  fn test_dup_with_positive_arg_and_too_small_stack() {
    let err = act_on_stack_err(&DupCommand, Some(3), vec![10, 20]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 3, actual: 2 },
    );
  }

  #[test]
  fn test_dup_with_positive_arg_on_empty_stack() {
    let err = act_on_stack_err(&DupCommand, Some(2), vec![]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 2, actual: 0 },
    );
  }

  #[test]
  fn test_dup_with_argument_one() {
    let output_stack = act_on_stack(&DupCommand, Some(1), vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 50]));
  }

  #[test]
  fn test_dup_with_argument_one_empty_stack() {
    let err = act_on_stack_err(&DupCommand, Some(1), vec![]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 1, actual: 0 },
    );
  }

  #[test]
  fn test_dup_with_argument_zero() {
    let output_stack = act_on_stack(&DupCommand, Some(0), vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 10, 20, 30, 40, 50]));
  }

  #[test]
  fn test_dup_argument_zero_on_empty_stack() {
    let output_stack = act_on_stack(&DupCommand, Some(0), vec![]);
    assert_eq!(output_stack, stack_of(vec![]));
  }

  #[test]
  fn test_dup_with_negative_one_arg() {
    let output_stack = act_on_stack(&DupCommand, Some(-1), vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 50]));
  }

  #[test]
  fn test_dup_with_negative_one_arg_on_stack_size_one() {
    let output_stack = act_on_stack(&DupCommand, Some(-1), vec![10]);
    assert_eq!(output_stack, stack_of(vec![10, 10]));
  }

  #[test]
  fn test_dup_with_negative_one_arg_on_empty_stack() {
    let err = act_on_stack_err(&DupCommand, Some(-1), vec![]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 1, actual: 0 },
    );
  }

  #[test]
  fn test_dup_with_negative_arg() {
    let output_stack = act_on_stack(&DupCommand, Some(-3), vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 30]));
  }

  #[test]
  fn test_dup_with_negative_arg_at_bottom_of_stack() {
    let output_stack = act_on_stack(&DupCommand, Some(-4), vec![10, 20, 30, 40]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 10]));
  }

  #[test]
  fn test_dup_with_negative_arg_and_too_small_stack() {
    let err = act_on_stack_err(&DupCommand, Some(-3), vec![10, 20]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 3, actual: 2 },
    )
  }

  #[test]
  fn test_dup_with_negative_arg_and_empty_stack() {
    let err = act_on_stack_err(&DupCommand, Some(-3), vec![]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 3, actual: 0 },
    )
  }
}
