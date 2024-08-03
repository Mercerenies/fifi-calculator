
//! Generic commands for manipulating data (usually vectors) in a
//! statistical fashion.

use crate::errorlist::ErrorList;
use crate::expr::Expr;
use crate::expr::vector::Vector;
use crate::stack::base::{StackLike, RandomAccessStackLike};
use crate::stack::keepable::KeepableStack;
use crate::state::ApplicationState;
use super::base::{Command, CommandContext, CommandOutput};
use super::arguments::{NullaryArgumentSchema, validate_schema};

use std::cmp::Ordering;

/// A `DatasetDrivenCommand`, by default, pops a single value off the
/// stack and calls a function on it, similar to
/// [`UnaryFunctionCommand`](super::functional::UnaryFunctionCommand).
/// However, the former's behavior in the presence of an explicit
/// numerical argument differs from `UnaryFunctionCommand`.
///
/// If given a positive numerical argument N, `DatasetDrivenCommand`
/// instead pops N values off the stack, groups them into a vector,
/// and then calls the function on that vector. A numerical argument
/// of zero pops the whole stack into a vector.
///
/// A negative numerical argument N pops a single value at position
/// (-N) on the stack. A numerical argument of -1 is equivalent to no
/// numerical argument at all.
///
/// In any case, the keep modifier is respected. In particular, in the
/// case of a negative numerical argument, the given stack position
/// will be duplicated in-place before modification.
pub struct DatasetDrivenCommand {
  function: Box<dyn Fn(Expr) -> Expr + Send + Sync>,
}

impl DatasetDrivenCommand {
  pub fn new<F>(function: F) -> Self
  where F: Fn(Expr) -> Expr + Send + Sync + 'static {
    Self { function: Box::new(function) }
  }

  pub fn named(function_name: impl Into<String>) -> Self {
    let function_name = function_name.into();
    Self::new(move |expr| Expr::Call(function_name.clone(), vec![expr]))
  }

  fn wrap_expr(&self, arg: Expr) -> Expr {
    (self.function)(arg)
  }

  fn group_and_apply_to_top(
    &self,
    state: &mut ApplicationState,
    element_count: usize,
    ctx: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    let calculation_mode = state.calculation_mode().clone();

    let mut errors = ErrorList::new();
    let values = {
      let mut stack = KeepableStack::new(state.main_stack_mut(), ctx.opts.keep_modifier);
      stack.pop_several(element_count)?
    };
    let values_as_vec = Vector::from(values);
    let expr = self.wrap_expr(values_as_vec.into());
    let expr = ctx.simplify_expr(expr, calculation_mode.clone(), &mut errors);
    state.main_stack_mut().push(expr);
    Ok(CommandOutput::from_errors(errors))
  }

  fn apply_to_single_element(
    &self,
    state: &mut ApplicationState,
    element_index: usize,
    ctx: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    let calculation_mode = state.calculation_mode().clone();

    let mut errors = ErrorList::new();
    let mut expr = state.main_stack_mut().pop_nth(element_index)?;
    if ctx.opts.keep_modifier {
      // expect safety: We just popped a value from that position, so
      // it's safe to re-insert.
      state.main_stack_mut().insert(element_index, expr.clone()).expect("Stack was too small for re-insert");
    }
    expr.mutate(|e| {
      let e = self.wrap_expr(e);
      ctx.simplify_expr(e, calculation_mode.clone(), &mut errors)
    });
    // expect safety: We just popped a value from that position, so
    // it's safe to re-insert.
    state.main_stack_mut().insert(element_index, expr).expect("Stack was too small for re-insert");
    Ok(CommandOutput::from_errors(errors))
  }
}

impl Command for DatasetDrivenCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    ctx: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    validate_schema(&NullaryArgumentSchema::new(), args)?;
    state.undo_stack_mut().push_cut();
    let arg = ctx.opts.argument.unwrap_or(-1);
    match arg.cmp(&0) {
      Ordering::Greater => {
        // Group top N elements and apply.
        self.group_and_apply_to_top(state, arg as usize, ctx)
      }
      Ordering::Less => {
        // Apply to single element N down on the stack.
        self.apply_to_single_element(state, (- arg - 1) as usize, ctx)
      }
      Ordering::Equal => {
        // Group all stack elements and apply.
        self.group_and_apply_to_top(state, state.main_stack().len(), ctx)
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::command::test_utils::act_on_stack;
  use crate::command::options::CommandOptions;
  use crate::stack::{Stack, StackError};

  fn dataset_command() -> DatasetDrivenCommand {
    DatasetDrivenCommand::named("test_func")
  }

  #[test]
  fn test_dataset_command() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&dataset_command(), (), input_stack).unwrap();
    assert_eq!(output_stack, Stack::from(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::from(30),
      Expr::call("test_func", vec![Expr::from(40)]),
    ]));
  }

  #[test]
  fn test_dataset_command_with_keep_arg() {
    let opts = CommandOptions::default().with_keep_modifier();
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&dataset_command(), opts, input_stack).unwrap();
    assert_eq!(output_stack, Stack::from(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::from(30),
      Expr::from(40),
      Expr::call("test_func", vec![Expr::from(40)]),
    ]));
  }

  #[test]
  fn test_dataset_command_on_empty_stack() {
    let input_stack = Vec::<Expr>::new();
    let error = act_on_stack(&dataset_command(), (), input_stack).unwrap_err();
    let error = error.downcast::<StackError>().unwrap();
    assert_eq!(error, StackError::NotEnoughElements { expected: 1, actual: 0 });
  }

  #[test]
  fn test_dataset_command_with_explicit_arg_minus_one() {
    let opts = CommandOptions::numerical(-1);
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&dataset_command(), opts, input_stack).unwrap();
    assert_eq!(output_stack, Stack::from(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::from(30),
      Expr::call("test_func", vec![Expr::from(40)]),
    ]));
  }

  #[test]
  fn test_dataset_command_with_keep_arg_and_explicit_arg_minus_one() {
    let opts = CommandOptions::numerical(-1).with_keep_modifier();
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&dataset_command(), opts, input_stack).unwrap();
    assert_eq!(output_stack, Stack::from(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::from(30),
      Expr::from(40),
      Expr::call("test_func", vec![Expr::from(40)]),
    ]));
  }

  #[test]
  fn test_dataset_command_with_negative_arg() {
    let opts = CommandOptions::numerical(-3);
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&dataset_command(), opts, input_stack).unwrap();
    assert_eq!(output_stack, Stack::from(vec![
      Expr::from(10),
      Expr::call("test_func", vec![Expr::from(20)]),
      Expr::from(30),
      Expr::from(40),
    ]));
  }

  #[test]
  fn test_dataset_command_with_keep_arg_and_negative_arg() {
    let opts = CommandOptions::numerical(-3).with_keep_modifier();
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&dataset_command(), opts, input_stack).unwrap();
    assert_eq!(output_stack, Stack::from(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("test_func", vec![Expr::from(20)]),
      Expr::from(30),
      Expr::from(40),
    ]));
  }

  #[test]
  fn test_dataset_command_with_negative_arg_bottom_of_stack() {
    let opts = CommandOptions::numerical(-4);
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&dataset_command(), opts, input_stack).unwrap();
    assert_eq!(output_stack, Stack::from(vec![
      Expr::call("test_func", vec![Expr::from(10)]),
      Expr::from(20),
      Expr::from(30),
      Expr::from(40),
    ]));
  }

  #[test]
  fn test_dataset_command_with_negative_arg_out_of_bounds() {
    let opts = CommandOptions::numerical(-5);
    let input_stack = vec![10, 20, 30, 40];
    let error = act_on_stack(&dataset_command(), opts, input_stack).unwrap_err();
    let error = error.downcast::<StackError>().unwrap();
    assert_eq!(error, StackError::NotEnoughElements { expected: 5, actual: 4 });
  }

  #[test]
  fn test_dataset_command_with_positive_one_arg() {
    let opts = CommandOptions::numerical(1);
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&dataset_command(), opts, input_stack).unwrap();
    assert_eq!(output_stack, Stack::from(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::from(30),
      Expr::call("test_func", vec![Expr::call("vector", vec![Expr::from(40)])]),
    ]));
  }

  #[test]
  fn test_dataset_command_with_positive_one_arg_and_keep_modifier() {
    let opts = CommandOptions::numerical(1).with_keep_modifier();
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&dataset_command(), opts, input_stack).unwrap();
    assert_eq!(output_stack, Stack::from(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::from(30),
      Expr::from(40),
      Expr::call("test_func", vec![Expr::call("vector", vec![Expr::from(40)])]),
    ]));
  }

  #[test]
  fn test_dataset_command_with_positive_arg() {
    let opts = CommandOptions::numerical(3);
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&dataset_command(), opts, input_stack).unwrap();
    assert_eq!(output_stack, Stack::from(vec![
      Expr::from(10),
      Expr::call("test_func", vec![Expr::call("vector", vec![
        Expr::from(20),
        Expr::from(30),
        Expr::from(40),
      ])]),
    ]));
  }

  #[test]
  fn test_dataset_command_with_positive_arg_and_keep_modifier() {
    let opts = CommandOptions::numerical(3).with_keep_modifier();
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&dataset_command(), opts, input_stack).unwrap();
    assert_eq!(output_stack, Stack::from(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::from(30),
      Expr::from(40),
      Expr::call("test_func", vec![Expr::call("vector", vec![
        Expr::from(20),
        Expr::from(30),
        Expr::from(40),
      ])]),
    ]));
  }

  #[test]
  fn test_dataset_command_with_positive_arg_whole_stack() {
    let opts = CommandOptions::numerical(4);
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&dataset_command(), opts, input_stack).unwrap();
    assert_eq!(output_stack, Stack::from(vec![
      Expr::call("test_func", vec![Expr::call("vector", vec![
        Expr::from(10),
        Expr::from(20),
        Expr::from(30),
        Expr::from(40),
      ])]),
    ]));
  }

  #[test]
  fn test_dataset_command_with_positive_arg_not_enough_elements() {
    let opts = CommandOptions::numerical(5);
    let input_stack = vec![10, 20, 30, 40];
    let error = act_on_stack(&dataset_command(), opts, input_stack).unwrap_err();
    let error = error.downcast::<StackError>().unwrap();
    assert_eq!(error, StackError::NotEnoughElements { expected: 5, actual: 4 });
  }

  #[test]
  fn test_dataset_command_with_zero_arg() {
    let opts = CommandOptions::numerical(0);
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&dataset_command(), opts, input_stack).unwrap();
    assert_eq!(output_stack, Stack::from(vec![
      Expr::call("test_func", vec![Expr::call("vector", vec![
        Expr::from(10),
        Expr::from(20),
        Expr::from(30),
        Expr::from(40),
      ])]),
    ]));
  }

  #[test]
  fn test_dataset_command_with_zero_arg_on_empty_stack() {
    let opts = CommandOptions::numerical(0);
    let input_stack = Vec::<Expr>::new();
    let output_stack = act_on_stack(&dataset_command(), opts, input_stack).unwrap();
    assert_eq!(output_stack, Stack::from(vec![
      Expr::call("test_func", vec![Expr::call("vector", vec![])]),
    ]));
  }

  #[test]
  fn test_dataset_command_with_zero_arg_and_keep_arg() {
    let opts = CommandOptions::numerical(0).with_keep_modifier();
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&dataset_command(), opts, input_stack).unwrap();
    assert_eq!(output_stack, Stack::from(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::from(30),
      Expr::from(40),
      Expr::call("test_func", vec![Expr::call("vector", vec![
        Expr::from(10),
        Expr::from(20),
        Expr::from(30),
        Expr::from(40),
      ])]),
    ]));
  }
}
