
//! Commands for shuffling the stack.

use super::base::{Command, CommandContext, CommandOutput};
use super::arguments::{NullaryArgumentSchema, validate_schema};
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
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    ctx: &CommandContext,
  ) -> Result<CommandOutput, Error> {
    // Note: PopCommand explicitly ignores the keep_modifier, as it
    // would always be a no-op.
    validate_schema(NullaryArgumentSchema::new(), args)?;
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
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    ctx: &CommandContext,
  ) -> Result<CommandOutput, Error> {
    validate_schema(NullaryArgumentSchema::new(), args)?;
    state.undo_stack_mut().push_cut();
    let mut stack = KeepableStack::new(state.main_stack_mut(), ctx.opts.keep_modifier);

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
        stack.check_stack_size((- arg) as usize)?;
        let mut all_elements = stack.pop_all();
        all_elements.rotate_right((- arg) as usize);
        stack.push_several(all_elements);
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
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    ctx: &CommandContext,
  ) -> Result<CommandOutput, Error> {
    // Note: DupCommand explicitly ignores the keep_modifier, as its
    // behavior would be quite unintuitive (especially with negative
    // numerical arg).
    validate_schema(NullaryArgumentSchema::new(), args)?;
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
  use crate::command::options::CommandOptions;
  use crate::stack::test_utils::stack_of;
  use crate::stack::StackError;

  #[test]
  fn test_simple_pop() {
    let output_stack = act_on_stack(&PopCommand, CommandOptions::default(), vec![10, 20, 30]);
    assert_eq!(output_stack, stack_of(vec![10, 20]));
  }

  #[test]
  fn test_simple_pop_with_keep_arg() {
    // keep_modifier has no effect on pop commands.
    let output_stack = act_on_stack(&PopCommand, CommandOptions::default().with_keep_modifier(), vec![10, 20, 30]);
    assert_eq!(output_stack, stack_of(vec![10, 20]));
  }

  #[test]
  fn test_simple_pop_on_empty_stack() {
    let err = act_on_stack_err(&PopCommand, CommandOptions::default(), vec![]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 1, actual: 0 },
    )
  }

  #[test]
  fn test_multiple_pop() {
    let output_stack = act_on_stack(
      &PopCommand,
      CommandOptions::numerical(3),
      vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![10, 20]));
  }

  #[test]
  fn test_multiple_pop_with_keep_arg() {
    // keep_modifier has no effect on pop commands.
    let output_stack = act_on_stack(
      &PopCommand,
      CommandOptions::numerical(3).with_keep_modifier(),
      vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![10, 20]));
  }

  #[test]
  fn test_multiple_pop_all_stack_elements() {
    let output_stack = act_on_stack(&PopCommand, CommandOptions::numerical(4), vec![10, 20, 30, 40]);
    assert_eq!(output_stack, stack_of(vec![]));
  }

  #[test]
  fn test_multiple_pop_on_empty_stack() {
    let err = act_on_stack_err(&PopCommand, CommandOptions::numerical(3), vec![]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 3, actual: 0 },
    );
  }

  #[test]
  fn test_multiple_pop_on_stack_thats_too_small() {
    let err = act_on_stack_err(&PopCommand, CommandOptions::numerical(4), vec![10, 20, 30]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 4, actual: 3 },
    );
  }

  #[test]
  fn test_pop_with_argument_zero() {
    let output_stack = act_on_stack(&PopCommand, CommandOptions::numerical(0), vec![10, 20, 30]);
    assert_eq!(output_stack, stack_of(vec![]));
  }

  #[test]
  fn test_pop_with_argument_zero_and_keep_arg() {
    // keep_modifier has no effect on pop commands.
    let output_stack = act_on_stack(&PopCommand, CommandOptions::numerical(0).with_keep_modifier(), vec![10, 20, 30]);
    assert_eq!(output_stack, stack_of(vec![]));
  }

  #[test]
  fn test_pop_with_argument_zero_on_empty_stack() {
    let output_stack = act_on_stack(&PopCommand, CommandOptions::numerical(0), vec![]);
    assert_eq!(output_stack, stack_of(vec![]));
  }

  #[test]
  fn test_pop_with_negative_one_argument() {
    let output_stack = act_on_stack(&PopCommand, CommandOptions::numerical(-1), vec![10, 20, 30, 40]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30]));
  }

  #[test]
  fn test_pop_with_negative_one_argument_and_empty_stack() {
    let err = act_on_stack_err(&PopCommand, CommandOptions::numerical(-1), vec![]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 1, actual: 0 },
    );
  }

  #[test]
  fn test_pop_with_negative_argument() {
    let output_stack = act_on_stack(&PopCommand, CommandOptions::numerical(-3), vec![10, 20, 30, 40]);
    assert_eq!(output_stack, stack_of(vec![10, 30, 40]));
  }

  #[test]
  fn test_pop_with_negative_argument_and_empty_stack() {
    let err = act_on_stack_err(&PopCommand, CommandOptions::numerical(-3), vec![]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 3, actual: 0 },
    );
  }

  #[test]
  fn test_pop_with_negative_argument_and_too_small_stack() {
    let err = act_on_stack_err(&PopCommand, CommandOptions::numerical(-3), vec![10, 20]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 3, actual: 2 },
    );
  }

  #[test]
  fn test_pop_with_negative_argument_at_bottom_of_stack() {
    let output_stack = act_on_stack(&PopCommand, CommandOptions::numerical(-4), vec![10, 20, 30, 40]);
    assert_eq!(output_stack, stack_of(vec![20, 30, 40]));
  }

  #[test]
  fn test_swap() {
    let output_stack = act_on_stack(&SwapCommand, CommandOptions::default(), vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 50, 40]));
  }

  #[test]
  fn test_swap_with_keep_arg() {
    let opts = CommandOptions::default().with_keep_modifier();
    let output_stack = act_on_stack(&SwapCommand, opts, vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 50, 40]));
  }

  #[test]
  fn test_swap_on_stack_size_two() {
    let output_stack = act_on_stack(&SwapCommand, CommandOptions::default(), vec![10, 20]);
    assert_eq!(output_stack, stack_of(vec![20, 10]));
  }

  #[test]
  fn test_swap_on_stack_size_one() {
    let err = act_on_stack_err(&SwapCommand, CommandOptions::default(), vec![10]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 2, actual: 1 },
    );
  }

  #[test]
  fn test_swap_on_stack_size_one_and_keep_arg() {
    // Keep modifier doesn't change anything in the case of an error.
    let opts = CommandOptions::default().with_keep_modifier();
    let err = act_on_stack_err(&SwapCommand, opts, vec![10]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 2, actual: 1 },
    );
  }

  #[test]
  fn test_swap_on_empty_stack() {
    let err = act_on_stack_err(&SwapCommand, CommandOptions::default(), vec![]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 2, actual: 0 },
    );
  }

  #[test]
  fn test_swap_positive_arg() {
    let output_stack = act_on_stack(&SwapCommand, CommandOptions::numerical(4), vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![10, 50, 20, 30, 40]));
  }

  #[test]
  fn test_swap_positive_arg_and_keep_arg() {
    let opts = CommandOptions::numerical(4).with_keep_modifier();
    let output_stack = act_on_stack(&SwapCommand, opts, vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 50, 20, 30, 40]));
  }

  #[test]
  fn test_swap_positive_arg_equal_to_stack_size() {
    let output_stack = act_on_stack(&SwapCommand, CommandOptions::numerical(3), vec![10, 20, 30]);
    assert_eq!(output_stack, stack_of(vec![30, 10, 20]));
  }

  #[test]
  fn test_swap_with_positive_arg_and_too_small_stack() {
    let err = act_on_stack_err(&SwapCommand, CommandOptions::numerical(3), vec![10, 20]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 3, actual: 2 },
    );
  }

  #[test]
  fn test_swap_with_positive_arg_on_empty_stack() {
    let err = act_on_stack_err(&SwapCommand, CommandOptions::numerical(4), vec![]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 4, actual: 0 },
    );
  }

  #[test]
  fn test_swap_arg_of_one() {
    let output_stack = act_on_stack(&SwapCommand, CommandOptions::numerical(1), vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50]));
  }

  #[test]
  fn test_swap_arg_of_one_and_keep_arg() {
    let opts = CommandOptions::numerical(1).with_keep_modifier();
    let output_stack = act_on_stack(&SwapCommand, opts, vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 50]));
  }

  #[test]
  fn test_swap_arg_of_one_on_empty_stack() {
    let err = act_on_stack_err(&SwapCommand, CommandOptions::numerical(1), vec![]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 1, actual: 0 },
    );
  }

  #[test]
  fn test_swap_argument_zero() {
    let output_stack = act_on_stack(&SwapCommand, CommandOptions::numerical(0), vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![50, 40, 30, 20, 10]));
  }

  #[test]
  fn test_swap_argument_zero_and_keep_arg() {
    let opts = CommandOptions::numerical(0).with_keep_modifier();
    let output_stack = act_on_stack(&SwapCommand, opts, vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 50, 40, 30, 20, 10]));
  }

  #[test]
  fn test_swap_argument_zero_on_stack_size_one() {
    let output_stack = act_on_stack(&SwapCommand, CommandOptions::numerical(0), vec![10]);
    assert_eq!(output_stack, stack_of(vec![10]));
  }

  #[test]
  fn test_swap_argument_zero_and_keep_arg_on_stack_size_one() {
    let opts = CommandOptions::numerical(0).with_keep_modifier();
    let output_stack = act_on_stack(&SwapCommand, opts, vec![10]);
    assert_eq!(output_stack, stack_of(vec![10, 10]));
  }

  #[test]
  fn test_swap_argument_zero_on_empty_stack() {
    let opts = CommandOptions::numerical(0);
    let output_stack = act_on_stack(&SwapCommand, opts, vec![]);
    assert_eq!(output_stack, stack_of(vec![]));
  }

  #[test]
  fn test_swap_argument_zero_with_keep_arg_on_empty_stack() {
    // Nothing to pop, so nothing for keep_modifier to preserve.
    // keep_modifier has no effect.
    let opts = CommandOptions::numerical(0).with_keep_modifier();
    let output_stack = act_on_stack(&SwapCommand, opts, vec![]);
    assert_eq!(output_stack, stack_of(vec![]));
  }

  #[test]
  fn test_swap_with_negative_one_arg() {
    let output_stack = act_on_stack(&SwapCommand, CommandOptions::numerical(-1), vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![50, 10, 20, 30, 40]));
  }

  #[test]
  fn test_swap_with_negative_one_arg_and_keep_modifier() {
    let opts = CommandOptions::numerical(-1).with_keep_modifier();
    let output_stack = act_on_stack(&SwapCommand, opts, vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 50, 10, 20, 30, 40]));
  }

  #[test]
  fn test_swap_with_negative_one_arg_on_stack_size_one() {
    let output_stack = act_on_stack(&SwapCommand, CommandOptions::numerical(-1), vec![10]);
    assert_eq!(output_stack, stack_of(vec![10]));
  }

  #[test]
  fn test_swap_with_negative_one_arg_on_empty_stack() {
    let err = act_on_stack_err(&SwapCommand, CommandOptions::numerical(-1), vec![]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 1, actual: 0 },
    );
  }

  #[test]
  fn test_swap_with_negative_arg() {
    let output_stack = act_on_stack(&SwapCommand, CommandOptions::numerical(-3), vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![30, 40, 50, 10, 20]));
  }

  #[test]
  fn test_swap_with_negative_arg_and_keep_arg() {
    let opts = CommandOptions::numerical(-3).with_keep_modifier();
    let output_stack = act_on_stack(&SwapCommand, opts, vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 30, 40, 50, 10, 20]));
  }

  #[test]
  fn test_swap_with_negative_arg_whole_stack() {
    let output_stack = act_on_stack(&SwapCommand, CommandOptions::numerical(-5), vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50]));
  }

  #[test]
  fn test_swap_with_negative_arg_and_too_small_stack() {
    let err = act_on_stack_err(&SwapCommand, CommandOptions::numerical(-3), vec![10, 20]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 3, actual: 2 },
    )
  }

  #[test]
  fn test_swap_with_negative_arg_and_empty_stack() {
    let err = act_on_stack_err(&SwapCommand, CommandOptions::numerical(-3), vec![]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 3, actual: 0 },
    )
  }

  #[test]
  fn test_dup() {
    let output_stack = act_on_stack(&DupCommand, CommandOptions::default(), vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 50]));
  }

  #[test]
  fn test_dup_with_keep_arg() {
    // keep_modifier has no effect on dup commands.
    let output_stack = act_on_stack(
      &DupCommand,
      CommandOptions::default().with_keep_modifier(),
      vec![10, 20, 30, 40, 50],
    );
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 50]));
  }

  #[test]
  fn test_dup_on_stack_size_one() {
    let output_stack = act_on_stack(&DupCommand, CommandOptions::default(), vec![10]);
    assert_eq!(output_stack, stack_of(vec![10, 10]));
  }

  #[test]
  fn test_dup_on_empty_stack() {
    let err = act_on_stack_err(&DupCommand, CommandOptions::default(), vec![]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 1, actual: 0 },
    );
  }

  #[test]
  fn test_dup_positive_arg() {
    let output_stack = act_on_stack(&DupCommand, CommandOptions::numerical(2), vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 40, 50]));
  }

  #[test]
  fn test_dup_positive_arg_equal_to_stack_size() {
    let output_stack = act_on_stack(&DupCommand, CommandOptions::numerical(3), vec![10, 20, 30]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 10, 20, 30]));
  }

  #[test]
  fn test_dup_with_positive_arg_and_too_small_stack() {
    let err = act_on_stack_err(&DupCommand, CommandOptions::numerical(3), vec![10, 20]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 3, actual: 2 },
    );
  }

  #[test]
  fn test_dup_with_positive_arg_on_empty_stack() {
    let err = act_on_stack_err(&DupCommand, CommandOptions::numerical(2), vec![]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 2, actual: 0 },
    );
  }

  #[test]
  fn test_dup_with_argument_one() {
    let output_stack = act_on_stack(&DupCommand, CommandOptions::numerical(1), vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 50]));
  }

  #[test]
  fn test_dup_with_argument_one_empty_stack() {
    let err = act_on_stack_err(&DupCommand, CommandOptions::numerical(1), vec![]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 1, actual: 0 },
    );
  }

  #[test]
  fn test_dup_with_argument_zero() {
    let output_stack = act_on_stack(&DupCommand, CommandOptions::numerical(0), vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 10, 20, 30, 40, 50]));
  }

  #[test]
  fn test_dup_argument_zero_on_empty_stack() {
    let output_stack = act_on_stack(&DupCommand, CommandOptions::numerical(0), vec![]);
    assert_eq!(output_stack, stack_of(vec![]));
  }

  #[test]
  fn test_dup_with_negative_one_arg() {
    let output_stack = act_on_stack(&DupCommand, CommandOptions::numerical(-1), vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 50]));
  }

  #[test]
  fn test_dup_with_negative_one_arg_on_stack_size_one() {
    let output_stack = act_on_stack(&DupCommand, CommandOptions::numerical(-1), vec![10]);
    assert_eq!(output_stack, stack_of(vec![10, 10]));
  }

  #[test]
  fn test_dup_with_negative_one_arg_on_empty_stack() {
    let err = act_on_stack_err(&DupCommand, CommandOptions::numerical(-1), vec![]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 1, actual: 0 },
    );
  }

  #[test]
  fn test_dup_with_negative_arg() {
    let output_stack = act_on_stack(&DupCommand, CommandOptions::numerical(-3), vec![10, 20, 30, 40, 50]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 30]));
  }

  #[test]
  fn test_dup_with_negative_arg_at_bottom_of_stack() {
    let output_stack = act_on_stack(&DupCommand, CommandOptions::numerical(-4), vec![10, 20, 30, 40]);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 10]));
  }

  #[test]
  fn test_dup_with_negative_arg_and_too_small_stack() {
    let err = act_on_stack_err(&DupCommand, CommandOptions::numerical(-3), vec![10, 20]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 3, actual: 2 },
    )
  }

  #[test]
  fn test_dup_with_negative_arg_and_empty_stack() {
    let err = act_on_stack_err(&DupCommand, CommandOptions::numerical(-3), vec![]);
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 3, actual: 0 },
    )
  }
}
