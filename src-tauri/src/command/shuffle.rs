
//! Commands for shuffling the stack.

use super::base::{Command, CommandContext, CommandOutput};
use super::arguments::{NullaryArgumentSchema, BinaryArgumentSchema, validate_schema};
use super::options::CommandOptions;
use super::subcommand::Subcommand;
use crate::state::ApplicationState;
use crate::stack::keepable::KeepableStack;
use crate::stack::base::{StackLike, RandomAccessStackLike};
use crate::expr::Expr;
use crate::expr::prisms::{StringToUsize, ParsedUsize};
use crate::util::prism::Identity;
use crate::mode::display::language::LanguageMode;
use crate::errorlist::ErrorList;

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

/// General-purpose re-order command. Takes two arguments: `src_pos`
/// and `dest_pos`. Both must be non-negative integers. The element at
/// stack position `src_pos` is popped and then inserted as index
/// `dest_pos`. If either index is out-of-bounds, nothing happens.
///
/// Note that `dest_pos` is indexed relative to the stack _after_ the
/// value is popped!
///
/// `MoveStackElemCommand` does not use the numerical argument. It
/// also does not respect the keep modifier, since no information is
/// being destroyed in this case.
#[derive(Debug, Clone)]
pub struct MoveStackElemCommand;

/// General-purpose replacement command. Takes two arguments: `pos`
/// and `value`. The former must be a non-negative integer, and the
/// latter will be parsed as an expression using the current language
/// mode.
///
/// If the index is out-of-bounds, nothing happens.
///
/// This command does not use the numerical argument, but it does
/// respect the "keep" modifier.
#[derive(Debug, Clone)]
pub struct ReplaceStackElemCommand {
  pub is_mouse_interaction: bool,
}

impl MoveStackElemCommand {
  fn argument_schema() -> BinaryArgumentSchema<StringToUsize, ParsedUsize, StringToUsize, ParsedUsize> {
    BinaryArgumentSchema::new(
      "nonnegative integer".to_owned(),
      StringToUsize,
      "nonnegative integer".to_owned(),
      StringToUsize,
    )
  }
}

impl ReplaceStackElemCommand {
  fn argument_schema() -> BinaryArgumentSchema<StringToUsize, ParsedUsize, Identity, String> {
    BinaryArgumentSchema::new(
      "nonnegative integer".to_owned(),
      StringToUsize,
      "any".to_owned(),
      Identity,
    )
  }

  fn try_parse(&self, arg: String, language_mode: &dyn LanguageMode) -> anyhow::Result<Expr> {
    language_mode.parse(&arg)
  }
}

impl Command for PopCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    ctx: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    // Note: PopCommand explicitly ignores the keep_modifier, as it
    // would always be a no-op.
    validate_schema(&NullaryArgumentSchema::new(), args)?;
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

  fn as_subcommand(&self, _opts: &CommandOptions) -> Option<Subcommand> {
    None
  }
}

impl Command for SwapCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    ctx: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    validate_schema(&NullaryArgumentSchema::new(), args)?;
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

  fn as_subcommand(&self, _opts: &CommandOptions) -> Option<Subcommand> {
    None
  }
}

impl Command for DupCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    ctx: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    // Note: DupCommand explicitly ignores the keep_modifier, as its
    // behavior would be quite unintuitive (especially with negative
    // numerical arg).
    validate_schema(&NullaryArgumentSchema::new(), args)?;
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

  fn as_subcommand(&self, _opts: &CommandOptions) -> Option<Subcommand> {
    None
  }
}

impl Command for MoveStackElemCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    _ctx: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    let (src_pos, dest_pos) = validate_schema(&Self::argument_schema(), args)?;
    state.undo_stack_mut().push_cut();
    let src_pos = usize::from(src_pos);
    let dest_pos = usize::from(dest_pos);

    if src_pos == dest_pos {
      // This command will do nothing, so exit early to make sure we
      // don't push anything silly onto the undo stack.
      return Ok(CommandOutput::success());
    }

    let mut stack = state.main_stack_mut();

    let required_stack_size = src_pos.max(dest_pos) + 1;
    stack.check_stack_size(required_stack_size)?;

    // We've validated the stack size, so we can assume future pushes
    // and pops will succeed.
    let value = stack.pop_nth(src_pos).expect("Stack underflow");
    stack.insert(dest_pos, value).expect("Stack insert out of bounds");

    Ok(
      CommandOutput::success()
        .set_force_scroll_down(false)
    )
  }

  fn as_subcommand(&self, _opts: &CommandOptions) -> Option<Subcommand> {
    None
  }
}
impl Command for ReplaceStackElemCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    ctx: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    let (index, value) = validate_schema(&Self::argument_schema(), args)?;

    let calculation_mode = state.calculation_mode().clone();

    let mut errors = ErrorList::new();
    let index = usize::from(index);
    state.undo_stack_mut().push_cut();

    let expr = self.try_parse(value, state.display_settings().language_mode().as_ref())?;
    let expr = ctx.simplify_expr(expr, calculation_mode, &mut errors);

    let mut stack = state.main_stack_mut();
    if ctx.opts.keep_modifier {
      // In principle, we're replacing the value at `index`, so there
      // must exist at least one extra value on the stack, even though
      // we don't use it since we have the keep modifier on.
      stack.check_stack_size(index + 1)?;
      stack.insert(index, expr)?;
    } else {
      let mut stack_elem = stack.get_mut(index as i64)?;
      *stack_elem = expr;
    }

    let mut output = CommandOutput::from_errors(errors);
    if self.is_mouse_interaction {
      output = output.set_force_scroll_down(false);
    }
    Ok(output)
  }

  fn as_subcommand(&self, _opts: &CommandOptions) -> Option<Subcommand> {
    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::command::test_utils::act_on_stack;
  use crate::command::options::CommandOptions;
  use crate::stack::test_utils::stack_of;
  use crate::stack::{Stack, StackError};
  use crate::expr::basic_parser::ParseError;

  #[test]
  fn test_simple_pop() {
    let output_stack = act_on_stack(&PopCommand, CommandOptions::default(), vec![10, 20, 30]).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 20]));
  }

  #[test]
  fn test_simple_pop_with_keep_arg() {
    // keep_modifier has no effect on pop commands.
    let output_stack = act_on_stack(&PopCommand, CommandOptions::default().with_keep_modifier(), vec![10, 20, 30]).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 20]));
  }

  #[test]
  fn test_simple_pop_on_empty_stack() {
    let err = act_on_stack(&PopCommand, CommandOptions::default(), Vec::<Expr>::new()).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
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
      vec![10, 20, 30, 40, 50],
    ).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 20]));
  }

  #[test]
  fn test_multiple_pop_with_keep_arg() {
    // keep_modifier has no effect on pop commands.
    let output_stack = act_on_stack(
      &PopCommand,
      CommandOptions::numerical(3).with_keep_modifier(),
      vec![10, 20, 30, 40, 50],
    ).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 20]));
  }

  #[test]
  fn test_multiple_pop_all_stack_elements() {
    let output_stack = act_on_stack(&PopCommand, CommandOptions::numerical(4), vec![10, 20, 30, 40]).unwrap();
    assert_eq!(output_stack, Stack::new());
  }

  #[test]
  fn test_multiple_pop_on_empty_stack() {
    let err = act_on_stack(&PopCommand, CommandOptions::numerical(3), Vec::<Expr>::new()).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 3, actual: 0 },
    );
  }

  #[test]
  fn test_multiple_pop_on_stack_thats_too_small() {
    let err = act_on_stack(&PopCommand, CommandOptions::numerical(4), vec![10, 20, 30]).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 4, actual: 3 },
    );
  }

  #[test]
  fn test_pop_with_argument_zero() {
    let output_stack = act_on_stack(&PopCommand, CommandOptions::numerical(0), vec![10, 20, 30]).unwrap();
    assert_eq!(output_stack, Stack::new());
  }

  #[test]
  fn test_pop_with_argument_zero_and_keep_arg() {
    // keep_modifier has no effect on pop commands.
    let output_stack = act_on_stack(
      &PopCommand,
      CommandOptions::numerical(0).with_keep_modifier(),
      vec![10, 20, 30],
    ).unwrap();
    assert_eq!(output_stack, Stack::new());
  }

  #[test]
  fn test_pop_with_argument_zero_on_empty_stack() {
    let output_stack = act_on_stack(&PopCommand, CommandOptions::numerical(0), Vec::<Expr>::new()).unwrap();
    assert_eq!(output_stack, Stack::new());
  }

  #[test]
  fn test_pop_with_negative_one_argument() {
    let output_stack = act_on_stack(&PopCommand, CommandOptions::numerical(-1), vec![10, 20, 30, 40]).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 20, 30]));
  }

  #[test]
  fn test_pop_with_negative_one_argument_and_empty_stack() {
    let err = act_on_stack(&PopCommand, CommandOptions::numerical(-1), Vec::<Expr>::new()).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 1, actual: 0 },
    );
  }

  #[test]
  fn test_pop_with_negative_argument() {
    let output_stack = act_on_stack(&PopCommand, CommandOptions::numerical(-3), vec![10, 20, 30, 40]).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 30, 40]));
  }

  #[test]
  fn test_pop_with_negative_argument_and_empty_stack() {
    let err = act_on_stack(&PopCommand, CommandOptions::numerical(-3), Vec::<Expr>::new()).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 3, actual: 0 },
    );
  }

  #[test]
  fn test_pop_with_negative_argument_and_too_small_stack() {
    let err = act_on_stack(&PopCommand, CommandOptions::numerical(-3), vec![10, 20]).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 3, actual: 2 },
    );
  }

  #[test]
  fn test_pop_with_negative_argument_at_bottom_of_stack() {
    let output_stack = act_on_stack(&PopCommand, CommandOptions::numerical(-4), vec![10, 20, 30, 40]).unwrap();
    assert_eq!(output_stack, stack_of(vec![20, 30, 40]));
  }

  #[test]
  fn test_swap() {
    let output_stack = act_on_stack(&SwapCommand, CommandOptions::default(), vec![10, 20, 30, 40, 50]).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 50, 40]));
  }

  #[test]
  fn test_swap_with_keep_arg() {
    let opts = CommandOptions::default().with_keep_modifier();
    let output_stack = act_on_stack(&SwapCommand, opts, vec![10, 20, 30, 40, 50]).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 50, 40]));
  }

  #[test]
  fn test_swap_on_stack_size_two() {
    let output_stack = act_on_stack(&SwapCommand, CommandOptions::default(), vec![10, 20]).unwrap();
    assert_eq!(output_stack, stack_of(vec![20, 10]));
  }

  #[test]
  fn test_swap_on_stack_size_one() {
    let err = act_on_stack(&SwapCommand, CommandOptions::default(), vec![10]).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 2, actual: 1 },
    );
  }

  #[test]
  fn test_swap_on_stack_size_one_and_keep_arg() {
    // Keep modifier doesn't change anything in the case of an error.
    let opts = CommandOptions::default().with_keep_modifier();
    let err = act_on_stack(&SwapCommand, opts, vec![10]).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 2, actual: 1 },
    );
  }

  #[test]
  fn test_swap_on_empty_stack() {
    let err = act_on_stack(&SwapCommand, CommandOptions::default(), Vec::<Expr>::new()).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 2, actual: 0 },
    );
  }

  #[test]
  fn test_swap_positive_arg() {
    let output_stack = act_on_stack(&SwapCommand, CommandOptions::numerical(4), vec![10, 20, 30, 40, 50]).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 50, 20, 30, 40]));
  }

  #[test]
  fn test_swap_positive_arg_and_keep_arg() {
    let opts = CommandOptions::numerical(4).with_keep_modifier();
    let output_stack = act_on_stack(&SwapCommand, opts, vec![10, 20, 30, 40, 50]).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 50, 20, 30, 40]));
  }

  #[test]
  fn test_swap_positive_arg_equal_to_stack_size() {
    let output_stack = act_on_stack(&SwapCommand, CommandOptions::numerical(3), vec![10, 20, 30]).unwrap();
    assert_eq!(output_stack, stack_of(vec![30, 10, 20]));
  }

  #[test]
  fn test_swap_with_positive_arg_and_too_small_stack() {
    let err = act_on_stack(&SwapCommand, CommandOptions::numerical(3), vec![10, 20]).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 3, actual: 2 },
    );
  }

  #[test]
  fn test_swap_with_positive_arg_on_empty_stack() {
    let err = act_on_stack(&SwapCommand, CommandOptions::numerical(4), Vec::<Expr>::new()).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 4, actual: 0 },
    );
  }

  #[test]
  fn test_swap_arg_of_one() {
    let output_stack = act_on_stack(&SwapCommand, CommandOptions::numerical(1), vec![10, 20, 30, 40, 50]).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50]));
  }

  #[test]
  fn test_swap_arg_of_one_and_keep_arg() {
    let opts = CommandOptions::numerical(1).with_keep_modifier();
    let output_stack = act_on_stack(&SwapCommand, opts, vec![10, 20, 30, 40, 50]).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 50]));
  }

  #[test]
  fn test_swap_arg_of_one_on_empty_stack() {
    let err = act_on_stack(&SwapCommand, CommandOptions::numerical(1), Vec::<Expr>::new()).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 1, actual: 0 },
    );
  }

  #[test]
  fn test_swap_argument_zero() {
    let output_stack = act_on_stack(&SwapCommand, CommandOptions::numerical(0), vec![10, 20, 30, 40, 50]).unwrap();
    assert_eq!(output_stack, stack_of(vec![50, 40, 30, 20, 10]));
  }

  #[test]
  fn test_swap_argument_zero_and_keep_arg() {
    let opts = CommandOptions::numerical(0).with_keep_modifier();
    let output_stack = act_on_stack(&SwapCommand, opts, vec![10, 20, 30, 40, 50]).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 50, 40, 30, 20, 10]));
  }

  #[test]
  fn test_swap_argument_zero_on_stack_size_one() {
    let output_stack = act_on_stack(&SwapCommand, CommandOptions::numerical(0), vec![10]).unwrap();
    assert_eq!(output_stack, stack_of(vec![10]));
  }

  #[test]
  fn test_swap_argument_zero_and_keep_arg_on_stack_size_one() {
    let opts = CommandOptions::numerical(0).with_keep_modifier();
    let output_stack = act_on_stack(&SwapCommand, opts, vec![10]).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 10]));
  }

  #[test]
  fn test_swap_argument_zero_on_empty_stack() {
    let opts = CommandOptions::numerical(0);
    let output_stack = act_on_stack(&SwapCommand, opts, Vec::<Expr>::new()).unwrap();
    assert_eq!(output_stack, Stack::new());
  }

  #[test]
  fn test_swap_argument_zero_with_keep_arg_on_empty_stack() {
    // Nothing to pop, so nothing for keep_modifier to preserve.
    // keep_modifier has no effect.
    let opts = CommandOptions::numerical(0).with_keep_modifier();
    let output_stack = act_on_stack(&SwapCommand, opts, Vec::<Expr>::new()).unwrap();
    assert_eq!(output_stack, Stack::new());
  }

  #[test]
  fn test_swap_with_negative_one_arg() {
    let output_stack = act_on_stack(&SwapCommand, CommandOptions::numerical(-1), vec![10, 20, 30, 40, 50]).unwrap();
    assert_eq!(output_stack, stack_of(vec![50, 10, 20, 30, 40]));
  }

  #[test]
  fn test_swap_with_negative_one_arg_and_keep_modifier() {
    let opts = CommandOptions::numerical(-1).with_keep_modifier();
    let output_stack = act_on_stack(&SwapCommand, opts, vec![10, 20, 30, 40, 50]).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 50, 10, 20, 30, 40]));
  }

  #[test]
  fn test_swap_with_negative_one_arg_on_stack_size_one() {
    let output_stack = act_on_stack(&SwapCommand, CommandOptions::numerical(-1), vec![10]).unwrap();
    assert_eq!(output_stack, stack_of(vec![10]));
  }

  #[test]
  fn test_swap_with_negative_one_arg_on_empty_stack() {
    let err = act_on_stack(&SwapCommand, CommandOptions::numerical(-1), Vec::<Expr>::new()).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 1, actual: 0 },
    );
  }

  #[test]
  fn test_swap_with_negative_arg() {
    let output_stack = act_on_stack(&SwapCommand, CommandOptions::numerical(-3), vec![10, 20, 30, 40, 50]).unwrap();
    assert_eq!(output_stack, stack_of(vec![30, 40, 50, 10, 20]));
  }

  #[test]
  fn test_swap_with_negative_arg_and_keep_arg() {
    let opts = CommandOptions::numerical(-3).with_keep_modifier();
    let output_stack = act_on_stack(&SwapCommand, opts, vec![10, 20, 30, 40, 50]).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 30, 40, 50, 10, 20]));
  }

  #[test]
  fn test_swap_with_negative_arg_whole_stack() {
    let output_stack = act_on_stack(&SwapCommand, CommandOptions::numerical(-5), vec![10, 20, 30, 40, 50]).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50]));
  }

  #[test]
  fn test_swap_with_negative_arg_and_too_small_stack() {
    let err = act_on_stack(&SwapCommand, CommandOptions::numerical(-3), vec![10, 20]).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 3, actual: 2 },
    )
  }

  #[test]
  fn test_swap_with_negative_arg_and_empty_stack() {
    let err = act_on_stack(&SwapCommand, CommandOptions::numerical(-3), Vec::<Expr>::new()).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 3, actual: 0 },
    )
  }

  #[test]
  fn test_dup() {
    let output_stack = act_on_stack(&DupCommand, CommandOptions::default(), vec![10, 20, 30, 40, 50]).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 50]));
  }

  #[test]
  fn test_dup_with_keep_arg() {
    // keep_modifier has no effect on dup commands.
    let output_stack = act_on_stack(
      &DupCommand,
      CommandOptions::default().with_keep_modifier(),
      vec![10, 20, 30, 40, 50],
    ).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 50]));
  }

  #[test]
  fn test_dup_on_stack_size_one() {
    let output_stack = act_on_stack(&DupCommand, CommandOptions::default(), vec![10]).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 10]));
  }

  #[test]
  fn test_dup_on_empty_stack() {
    let err = act_on_stack(&DupCommand, CommandOptions::default(), Vec::<Expr>::new()).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 1, actual: 0 },
    );
  }

  #[test]
  fn test_dup_positive_arg() {
    let output_stack = act_on_stack(&DupCommand, CommandOptions::numerical(2), vec![10, 20, 30, 40, 50]).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 40, 50]));
  }

  #[test]
  fn test_dup_positive_arg_equal_to_stack_size() {
    let output_stack = act_on_stack(&DupCommand, CommandOptions::numerical(3), vec![10, 20, 30]).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 10, 20, 30]));
  }

  #[test]
  fn test_dup_with_positive_arg_and_too_small_stack() {
    let err = act_on_stack(&DupCommand, CommandOptions::numerical(3), vec![10, 20]).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 3, actual: 2 },
    );
  }

  #[test]
  fn test_dup_with_positive_arg_on_empty_stack() {
    let err = act_on_stack(&DupCommand, CommandOptions::numerical(2), Vec::<Expr>::new()).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 2, actual: 0 },
    );
  }

  #[test]
  fn test_dup_with_argument_one() {
    let output_stack = act_on_stack(&DupCommand, CommandOptions::numerical(1), vec![10, 20, 30, 40, 50]).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 50]));
  }

  #[test]
  fn test_dup_with_argument_one_empty_stack() {
    let err = act_on_stack(&DupCommand, CommandOptions::numerical(1), Vec::<Expr>::new()).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 1, actual: 0 },
    );
  }

  #[test]
  fn test_dup_with_argument_zero() {
    let output_stack = act_on_stack(&DupCommand, CommandOptions::numerical(0), vec![10, 20, 30, 40, 50]).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 10, 20, 30, 40, 50]));
  }

  #[test]
  fn test_dup_argument_zero_on_empty_stack() {
    let output_stack = act_on_stack(&DupCommand, CommandOptions::numerical(0), Vec::<Expr>::new()).unwrap();
    assert_eq!(output_stack, Stack::new());
  }

  #[test]
  fn test_dup_with_negative_one_arg() {
    let output_stack = act_on_stack(&DupCommand, CommandOptions::numerical(-1), vec![10, 20, 30, 40, 50]).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 50]));
  }

  #[test]
  fn test_dup_with_negative_one_arg_on_stack_size_one() {
    let output_stack = act_on_stack(&DupCommand, CommandOptions::numerical(-1), vec![10]).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 10]));
  }

  #[test]
  fn test_dup_with_negative_one_arg_on_empty_stack() {
    let err = act_on_stack(&DupCommand, CommandOptions::numerical(-1), Vec::<Expr>::new()).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 1, actual: 0 },
    );
  }

  #[test]
  fn test_dup_with_negative_arg() {
    let output_stack = act_on_stack(&DupCommand, CommandOptions::numerical(-3), vec![10, 20, 30, 40, 50]).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 50, 30]));
  }

  #[test]
  fn test_dup_with_negative_arg_at_bottom_of_stack() {
    let output_stack = act_on_stack(&DupCommand, CommandOptions::numerical(-4), vec![10, 20, 30, 40]).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 10]));
  }

  #[test]
  fn test_dup_with_negative_arg_and_too_small_stack() {
    let err = act_on_stack(&DupCommand, CommandOptions::numerical(-3), vec![10, 20]).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 3, actual: 2 },
    )
  }

  #[test]
  fn test_dup_with_negative_arg_and_empty_stack() {
    let err = act_on_stack(&DupCommand, CommandOptions::numerical(-3), Vec::<Expr>::new()).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 3, actual: 0 },
    )
  }

  #[test]
  fn test_move_stack() {
    let input_stack = vec![10, 20, 30, 40, 50, 60, 70];
    let output_stack = act_on_stack(
      &MoveStackElemCommand,
      vec!["1", "3"],
      input_stack,
    ).unwrap();
    assert_eq!(
      output_stack,
      stack_of(vec![10, 20, 30, 60, 40, 50, 70]),
    )
  }

  #[test]
  fn test_move_stack_move_from_top() {
    let input_stack = vec![10, 20, 30, 40, 50, 60, 70];
    let output_stack = act_on_stack(
      &MoveStackElemCommand,
      vec!["0", "3"],
      input_stack,
    ).unwrap();
    assert_eq!(
      output_stack,
      stack_of(vec![10, 20, 30, 70, 40, 50, 60]),
    )
  }

  #[test]
  fn test_move_stack_move_from_bottom() {
    let input_stack = vec![10, 20, 30, 40, 50, 60, 70];
    let output_stack = act_on_stack(
      &MoveStackElemCommand,
      vec!["6", "2"],
      input_stack,
    ).unwrap();
    assert_eq!(
      output_stack,
      stack_of(vec![20, 30, 40, 50, 10, 60, 70]),
    )
  }

  #[test]
  fn test_move_stack_move_to_top() {
    let input_stack = vec![10, 20, 30, 40, 50, 60, 70];
    let output_stack = act_on_stack(
      &MoveStackElemCommand,
      vec!["1", "0"],
      input_stack,
    ).unwrap();
    assert_eq!(
      output_stack,
      stack_of(vec![10, 20, 30, 40, 50, 70, 60]),
    )
  }

  #[test]
  fn test_move_stack_move_to_bottom() {
    let input_stack = vec![10, 20, 30, 40, 50, 60, 70];
    let output_stack = act_on_stack(
      &MoveStackElemCommand,
      vec!["1", "6"],
      input_stack,
    ).unwrap();
    assert_eq!(
      output_stack,
      stack_of(vec![60, 10, 20, 30, 40, 50, 70]),
    )
  }

  #[test]
  fn test_move_stack_move_noop() {
    let input_stack = vec![10, 20, 30, 40, 50, 60, 70];
    let output_stack = act_on_stack(
      &MoveStackElemCommand,
      vec!["4", "4"],
      input_stack,
    ).unwrap();
    assert_eq!(
      output_stack,
      stack_of(vec![10, 20, 30, 40, 50, 60, 70]),
    )
  }

  #[test]
  fn test_move_stack_src_out_of_bounds() {
    let input_stack = vec![10, 20, 30, 40, 50, 60, 70];
    let err = act_on_stack(
      &MoveStackElemCommand,
      vec!["7", "1"],
      input_stack,
    ).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(err, StackError::NotEnoughElements { expected: 8, actual: 7 });
  }

  #[test]
  fn test_move_stack_dest_out_of_bounds() {
    let input_stack = vec![10, 20, 30, 40, 50, 60, 70];
    let err = act_on_stack(
      &MoveStackElemCommand,
      vec!["3", "7"],
      input_stack,
    ).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(err, StackError::NotEnoughElements { expected: 8, actual: 7 });
  }

  #[test]
  fn test_move_stack_both_out_of_bounds_1() {
    let input_stack = vec![10, 20, 30, 40, 50, 60, 70];
    let err = act_on_stack(
      &MoveStackElemCommand,
      vec!["8", "9"],
      input_stack,
    ).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(err, StackError::NotEnoughElements { expected: 10, actual: 7 });
  }

  #[test]
  fn test_move_stack_both_out_of_bounds_2() {
    let input_stack = vec![10, 20, 30, 40, 50, 60, 70];
    let err = act_on_stack(
      &MoveStackElemCommand,
      vec!["9", "8"],
      input_stack,
    ).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(err, StackError::NotEnoughElements { expected: 10, actual: 7 });
  }

  #[test]
  fn test_replace_stack_elem() {
    let input_stack = vec![10, 20, 30, 40, 50, 60, 70];
    let output_stack = act_on_stack(
      &ReplaceStackElemCommand { is_mouse_interaction: false },
      vec!["2", "99"],
      input_stack,
    ).unwrap();
    assert_eq!(
      output_stack,
      stack_of(vec![10, 20, 30, 40, 99, 60, 70]),
    )
  }

  #[test]
  fn test_replace_stack_elem_parse_failure() {
    let input_stack = vec![10, 20, 30, 40, 50, 60, 70];
    let err = act_on_stack(
      &ReplaceStackElemCommand { is_mouse_interaction: false },
      vec!["2", "("],
      input_stack,
    ).unwrap_err();
    assert!(err.is::<ParseError>(), "Expected ParseError, got {:?}", err);
  }

  #[test]
  fn test_replace_stack_elem_with_keep_arg() {
    let input_stack = vec![10, 20, 30, 40, 50, 60, 70];
    let output_stack = act_on_stack(
      &ReplaceStackElemCommand { is_mouse_interaction: false },
      (vec!["2", "99"], CommandOptions::default().with_keep_modifier()),
      input_stack,
    ).unwrap();
    assert_eq!(
      output_stack,
      stack_of(vec![10, 20, 30, 40, 50, 99, 60, 70]),
    )
  }

  #[test]
  fn test_replace_stack_elem_with_keep_arg_at_top_of_stack() {
    let input_stack = vec![10, 20, 30, 40, 50, 60, 70];
    let output_stack = act_on_stack(
      &ReplaceStackElemCommand { is_mouse_interaction: false },
      (vec!["0", "99"], CommandOptions::default().with_keep_modifier()),
      input_stack,
    ).unwrap();
    assert_eq!(
      output_stack,
      stack_of(vec![10, 20, 30, 40, 50, 60, 70, 99]),
    )
  }

  #[test]
  fn test_replace_stack_elem_with_keep_arg_at_bottom_of_stack() {
    let input_stack = vec![10, 20, 30, 40, 50, 60, 70];
    let output_stack = act_on_stack(
      &ReplaceStackElemCommand { is_mouse_interaction: false },
      (vec!["6", "99"], CommandOptions::default().with_keep_modifier()),
      input_stack,
    ).unwrap();
    assert_eq!(
      output_stack,
      stack_of(vec![10, 99, 20, 30, 40, 50, 60, 70]),
    )
  }

  #[test]
  fn test_replace_stack_elem_with_keep_arg_out_of_bounds() {
    let input_stack = vec![10, 20, 30, 40, 50, 60, 70];
    let err = act_on_stack(
      &ReplaceStackElemCommand { is_mouse_interaction: false },
      (vec!["7", "99"], CommandOptions::default().with_keep_modifier()),
      input_stack,
    ).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(
      err,
      StackError::NotEnoughElements { expected: 8, actual: 7 },
    )
  }
}
