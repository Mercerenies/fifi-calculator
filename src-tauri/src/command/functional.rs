
//! Commands that push functions onto the stack, using zero or more
//! arguments from the existing stack.

use super::base::{Command, CommandContext, CommandOutput};
use super::arguments::{NullaryArgumentSchema, validate_schema};
use crate::state::ApplicationState;
use crate::error::Error;
use crate::expr::Expr;
use crate::errorlist::ErrorList;
use crate::stack::base::StackLike;
use crate::stack::keepable::KeepableStack;

use std::cmp::Ordering;

/// A command that pushes a clone of its owned expression onto the
/// stack.
///
/// With a nonnegative prefix argument N, the expression is pushed
/// onto the stack N times. A negative prefix argument is treated as
/// zero.
#[derive(Clone, Debug)]
pub struct PushConstantCommand {
  expr: Expr,
}

/// A command that applies an arbitrary function to the top element of
/// the stack.
///
/// With a positive prefix argument N, applies the function
/// (separately) to the top N elements. A prefix argument of zero is
/// treated as being equal to the length of the stack.
///
/// With a negative prefix argument N, the function is applied to a
/// single element (-N) elements down on the stack. That is, a prefix
/// argument of -1 is equivalent to no argument at all, and a prefix
/// argument of -2 applies the function to the element just below the
/// top of the stack.
pub struct UnaryFunctionCommand {
  function: Box<dyn Fn(Expr, &ApplicationState) -> Expr + Send + Sync>,
}

/// A command that applies an arbitrary function to the top two
/// elements of the stack, with the second-from-the-top element of the
/// stack being the first argument to the function.
///
/// With a positive prefix argument N, the top N stack elements are
/// reduced down to one by repeatedly applying the function,
/// associating to the left. A prefix argument of zero reduces the
/// whole stack in this way.
///
/// A negative prefix argument N is a bit more complex. It pops one
/// element and then applies the binary function `function` to the top
/// (-N) elements of the remaining stack, with the second argument
/// specialized to the popped value.
pub struct BinaryFunctionCommand {
  function: Box<dyn Fn(Expr, Expr) -> Expr + Send + Sync>,
}

impl PushConstantCommand {
  pub fn new(expr: Expr) -> PushConstantCommand {
    PushConstantCommand { expr }
  }
}

impl UnaryFunctionCommand {
  pub fn new<F>(function: F) -> UnaryFunctionCommand
  where F: Fn(Expr) -> Expr + Send + Sync + 'static {
    UnaryFunctionCommand::with_state(move |arg, _| function(arg))
  }

  pub fn with_state<F>(function: F) -> UnaryFunctionCommand
  where F: Fn(Expr, &ApplicationState) -> Expr + Send + Sync + 'static {
    UnaryFunctionCommand { function: Box::new(function) }
  }

  pub fn named(function_name: impl Into<String>) -> UnaryFunctionCommand {
    let function_name = function_name.into();
    UnaryFunctionCommand::new(move |arg| {
      Expr::Call(function_name.clone(), vec![arg])
    })
  }

  fn wrap_expr(&self, arg: Expr, state: &ApplicationState) -> Expr {
    (self.function)(arg, state)
  }

  fn apply_to_top(
    &self,
    state: &mut ApplicationState,
    element_count: usize,
    ctx: &CommandContext,
  ) -> Result<CommandOutput, Error> {
    let mut errors = ErrorList::new();
    let values = {
      let mut stack = KeepableStack::new(state.main_stack_mut(), ctx.opts.keep_modifier);
      stack.pop_several(element_count as usize)?
    };
    let values: Vec<_> = values.into_iter().map(|e| {
      ctx.simplifier.simplify_expr(self.wrap_expr(e, state), &mut errors)
    }).collect();
    state.main_stack_mut().push_several(values);
    Ok(CommandOutput::from_errors(errors))
  }

  fn apply_to_single_element(
    &self,
    state: &mut ApplicationState,
    element_index: usize,
    ctx: &CommandContext,
  ) -> Result<CommandOutput, Error> {
    let mut errors = ErrorList::new();
    let mut expr = state.main_stack_mut().pop_nth(element_index)?;
    if ctx.opts.keep_modifier {
      // expect safety: We just popped a value from that position, so
      // it's safe to re-insert.
      state.main_stack_mut().insert(element_index, expr.clone()).expect("Stack was too small for re-insert");
    }
    expr.mutate(|e| ctx.simplifier.simplify_expr(self.wrap_expr(e, state), &mut errors));
    // expect safety: We just popped a value from that position, so
    // it's safe to re-insert.
    state.main_stack_mut().insert(element_index, expr).expect("Stack was too small for re-insert");
    Ok(CommandOutput::from_errors(errors))
  }
}

impl BinaryFunctionCommand {
  pub fn new<F>(function: F) -> BinaryFunctionCommand
  where F: Fn(Expr, Expr) -> Expr + Send + Sync + 'static {
    BinaryFunctionCommand { function: Box::new(function) }
  }

  pub fn named(function_name: impl Into<String>) -> BinaryFunctionCommand {
    let function_name = function_name.into();
    BinaryFunctionCommand::new(move |arg1, arg2| {
      Expr::Call(function_name.clone(), vec![arg1, arg2])
    })
  }

  fn wrap_exprs(&self, a: Expr, b: Expr) -> Expr {
    (self.function)(a, b)
  }
}

impl Command for PushConstantCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    ctx: &CommandContext,
  ) -> Result<CommandOutput, Error> {
    // Note: keep_modifier has no effect on this command (since there
    // are no pops), so we don't construct a KeepableStack.
    validate_schema(NullaryArgumentSchema::new(), args)?;
    state.undo_stack_mut().push_cut();
    let arg = ctx.opts.argument.unwrap_or(1).max(0);
    let mut errors = ErrorList::new();
    for _ in 0..arg {
      state.main_stack_mut().push(ctx.simplifier.simplify_expr(self.expr.clone(), &mut errors));
    }
    Ok(CommandOutput::from_errors(errors))
  }
}

impl Command for UnaryFunctionCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    ctx: &CommandContext,
  ) -> Result<CommandOutput, Error> {
    validate_schema(NullaryArgumentSchema::new(), args)?;
    state.undo_stack_mut().push_cut();
    let arg = ctx.opts.argument.unwrap_or(1);
    match arg.cmp(&0) {
      Ordering::Greater => {
        // Apply to top N elements.
        self.apply_to_top(state, arg as usize, ctx)
      }
      Ordering::Less => {
        // Apply to single element N down on the stack.
        self.apply_to_single_element(state, (- arg - 1) as usize, ctx)
      }
      Ordering::Equal => {
        // Apply to all elements.
        let stack_len = state.main_stack_mut().len();
        self.apply_to_top(state, stack_len, ctx)
      }
    }
  }
}

impl Command for BinaryFunctionCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    ctx: &CommandContext,
  ) -> Result<CommandOutput, Error> {
    validate_schema(NullaryArgumentSchema::new(), args)?;
    state.undo_stack_mut().push_cut();
    let mut stack = KeepableStack::new(state.main_stack_mut(), ctx.opts.keep_modifier);

    let mut errors = ErrorList::new();
    let arg = ctx.opts.argument.unwrap_or(2);
    match arg.cmp(&0) {
      Ordering::Greater => {
        // Perform reduction on top N elements.
        let values = stack.pop_several(arg as usize)?;
        let result = values.into_iter().reduce(|a, b| {
          self.wrap_exprs(a, b)
        }).expect("Empty stack"); // expect safety: We popped at least one element off the stack
        let result = ctx.simplifier.simplify_expr(result, &mut errors);
        stack.push(result);
      }
      Ordering::Less => {
        // Apply top to next N elements.
        let mut values = stack.pop_several((- arg + 1) as usize)?;
        // expect safety: We popped at least two values, so removing one is safe.
        let second_argument = values.pop().expect("Empty stack");
        for e in values.iter_mut() {
          e.mutate(|e| ctx.simplifier.simplify_expr(self.wrap_exprs(e, second_argument.clone()), &mut errors));
        }
        stack.push_several(values);
      }
      Ordering::Equal => {
        // Reduce entire stack.
        stack.check_stack_size(1)?;
        let values = stack.pop_all();
        let result = values.into_iter().reduce(|a, b| {
          self.wrap_exprs(a, b)
        }).expect("Empty stack"); // expect safety: We just checked that the stack was non-empty
        let result = ctx.simplifier.simplify_expr(result, &mut errors);
        stack.push(result);
      }
    }
    Ok(CommandOutput::from_errors(errors))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::command::test_utils::{act_on_stack, act_on_stack_err};
  use crate::command::options::CommandOptions;
  use crate::stack::test_utils::stack_of;
  use crate::stack::{Stack, StackError};
  use crate::expr::number::Number;

  fn push_constant_zero() -> PushConstantCommand {
    PushConstantCommand::new(Expr::from(Number::from(0)))
  }

  fn unary_function() -> UnaryFunctionCommand {
    UnaryFunctionCommand::named("test_func")
  }

  fn binary_function() -> BinaryFunctionCommand {
    BinaryFunctionCommand::named("test_func")
  }

  #[test]
  fn test_push_constant() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&push_constant_zero(), CommandOptions::default(), input_stack);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 0]));
  }

  #[test]
  fn test_push_constant_with_keep_arg() {
    // keep_modifier has no effect on push_constant.
    let opts = CommandOptions::default().with_keep_modifier();
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&push_constant_zero(), opts, input_stack);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 0]));
  }

  #[test]
  fn test_push_constant_with_empty_stack() {
    let input_stack = vec![];
    let output_stack = act_on_stack(&push_constant_zero(), CommandOptions::default(), input_stack);
    assert_eq!(output_stack, stack_of(vec![0]));
  }

  #[test]
  fn test_push_constant_with_explicit_argument_one() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&push_constant_zero(), CommandOptions::numerical(1), input_stack);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 0]));
  }

  #[test]
  fn test_push_constant_with_positive_argument() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&push_constant_zero(), CommandOptions::numerical(3), input_stack);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 0, 0, 0]));
  }

  #[test]
  fn test_push_constant_with_negative_argument() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&push_constant_zero(), CommandOptions::numerical(-4), input_stack);
    // Does not change the stack
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40]));
  }

  #[test]
  fn test_push_constant_with_argument_of_zero() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&push_constant_zero(), CommandOptions::numerical(0), input_stack);
    // Does not change the stack
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40]));
  }

  #[test]
  fn test_push_constant_with_argument_of_zero_and_keep_arg() {
    let opts = CommandOptions::numerical(0).with_keep_modifier();
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&push_constant_zero(), opts, input_stack);
    // Does not change the stack
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40]));
  }

  #[test]
  fn test_unary_function_command() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&unary_function(), CommandOptions::default(), input_stack);
    assert_eq!(
      output_stack,
      Stack::from(vec![
        Expr::from(10),
        Expr::from(20),
        Expr::from(30),
        Expr::call("test_func", vec![Expr::from(40)]),
      ]),
    );
  }

  #[test]
  fn test_unary_function_command_with_keep_arg() {
    let opts = CommandOptions::default().with_keep_modifier();
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&unary_function(), opts, input_stack);
    assert_eq!(
      output_stack,
      Stack::from(vec![
        Expr::from(10),
        Expr::from(20),
        Expr::from(30),
        Expr::from(40),
        Expr::call("test_func", vec![Expr::from(40)]),
      ]),
    );
  }

  #[test]
  fn test_unary_function_command_on_empty_stack() {
    let input_stack = vec![];
    let error = act_on_stack_err(&unary_function(), CommandOptions::default(), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 1, actual: 0 },
    );
  }

  #[test]
  fn test_unary_function_command_with_explicit_arg_one() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&unary_function(), CommandOptions::numerical(1), input_stack);
    assert_eq!(
      output_stack,
      Stack::from(vec![
        Expr::from(10),
        Expr::from(20),
        Expr::from(30),
        Expr::call("test_func", vec![Expr::from(40)]),
      ]),
    );
  }

  #[test]
  fn test_unary_function_command_with_arg_one_on_empty_stack() {
    let input_stack = vec![];
    let error = act_on_stack_err(&unary_function(), CommandOptions::numerical(1), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 1, actual: 0 },
    );
  }

  #[test]
  fn test_unary_function_command_with_arg_two() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&unary_function(), CommandOptions::numerical(2), input_stack);
    assert_eq!(
      output_stack,
      Stack::from(vec![
        Expr::from(10),
        Expr::from(20),
        Expr::call("test_func", vec![Expr::from(30)]),
        Expr::call("test_func", vec![Expr::from(40)]),
      ]),
    );
  }

  #[test]
  fn test_unary_function_command_with_arg_two_and_keep_arg() {
    let opts = CommandOptions::numerical(2).with_keep_modifier();
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&unary_function(), opts, input_stack);
    assert_eq!(
      output_stack,
      Stack::from(vec![
        Expr::from(10),
        Expr::from(20),
        Expr::from(30),
        Expr::from(40),
        Expr::call("test_func", vec![Expr::from(30)]),
        Expr::call("test_func", vec![Expr::from(40)]),
      ]),
    );
  }

  #[test]
  fn test_unary_function_command_with_arg_two_on_empty_stack() {
    let input_stack = vec![];
    let error = act_on_stack_err(&unary_function(), CommandOptions::numerical(2), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 2, actual: 0 },
    );
  }

  #[test]
  fn test_unary_function_command_with_arg_two_on_stack_size_one() {
    let input_stack = vec![10];
    let error = act_on_stack_err(&unary_function(), CommandOptions::numerical(2), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 2, actual: 1 },
    );
  }

  #[test]
  fn test_unary_function_command_with_arg_two_and_keep_arg_on_stack_size_one() {
    // keep_modifier has no effect when the stack is too small.
    let opts = CommandOptions::numerical(2).with_keep_modifier();
    let input_stack = vec![10];
    let error = act_on_stack_err(&unary_function(), opts, input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 2, actual: 1 },
    );
  }

  #[test]
  fn test_unary_function_command_with_arg_zero() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&unary_function(), CommandOptions::numerical(0), input_stack);
    assert_eq!(
      output_stack,
      Stack::from(vec![
        Expr::call("test_func", vec![Expr::from(10)]),
        Expr::call("test_func", vec![Expr::from(20)]),
        Expr::call("test_func", vec![Expr::from(30)]),
        Expr::call("test_func", vec![Expr::from(40)]),
      ]),
    );
  }

  #[test]
  fn test_unary_function_command_with_arg_zero_and_keep_arg() {
    let opts = CommandOptions::numerical(0).with_keep_modifier();
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&unary_function(), opts, input_stack);
    assert_eq!(
      output_stack,
      Stack::from(vec![
        Expr::from(10),
        Expr::from(20),
        Expr::from(30),
        Expr::from(40),
        Expr::call("test_func", vec![Expr::from(10)]),
        Expr::call("test_func", vec![Expr::from(20)]),
        Expr::call("test_func", vec![Expr::from(30)]),
        Expr::call("test_func", vec![Expr::from(40)]),
      ]),
    );
  }

  #[test]
  fn test_unary_function_command_with_arg_zero_on_empty_stack() {
    let input_stack = vec![];
    let output_stack = act_on_stack(&unary_function(), CommandOptions::numerical(0), input_stack);
    assert_eq!(output_stack, stack_of(vec![]));
  }

  #[test]
  fn test_unary_function_command_with_arg_zero_and_keep_arg_on_empty_stack() {
    // keep_modifier has no effect, since there's nothing to preserve.
    let opts = CommandOptions::numerical(0).with_keep_modifier();
    let input_stack = vec![];
    let output_stack = act_on_stack(&unary_function(), opts, input_stack);
    assert_eq!(output_stack, stack_of(vec![]));
  }

  #[test]
  fn test_unary_function_command_with_arg_negative_one() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&unary_function(), CommandOptions::numerical(-1), input_stack);
    assert_eq!(
      output_stack,
      Stack::from(vec![
        Expr::from(10),
        Expr::from(20),
        Expr::from(30),
        Expr::call("test_func", vec![Expr::from(40)]),
      ]),
    );
  }

  #[test]
  fn test_unary_function_command_with_arg_negative_one_and_keep_arg() {
    let opts = CommandOptions::numerical(-1).with_keep_modifier();
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&unary_function(), opts, input_stack);
    assert_eq!(
      output_stack,
      Stack::from(vec![
        Expr::from(10),
        Expr::from(20),
        Expr::from(30),
        Expr::from(40),
        Expr::call("test_func", vec![Expr::from(40)]),
      ]),
    );
  }

  #[test]
  fn test_unary_function_command_with_arg_negative_one_on_empty_stack() {
    let input_stack = vec![];
    let error = act_on_stack_err(&unary_function(), CommandOptions::numerical(1), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 1, actual: 0 },
    );
  }

  #[test]
  fn test_unary_function_command_with_arg_negative_two() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&unary_function(), CommandOptions::numerical(-2), input_stack);
    assert_eq!(
      output_stack,
      Stack::from(vec![
        Expr::from(10),
        Expr::from(20),
        Expr::call("test_func", vec![Expr::from(30)]),
        Expr::from(40),
      ]),
    );
  }

  #[test]
  fn test_unary_function_command_with_arg_negative_two_and_keep_arg() {
    let opts = CommandOptions::numerical(-2).with_keep_modifier();
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&unary_function(), opts, input_stack);
    assert_eq!(
      output_stack,
      Stack::from(vec![
        Expr::from(10),
        Expr::from(20),
        Expr::from(30),
        Expr::call("test_func", vec![Expr::from(30)]),
        Expr::from(40),
      ]),
    );
  }

  #[test]
  fn test_unary_function_command_with_arg_negative_two_on_empty_stack() {
    let input_stack = vec![];
    let error = act_on_stack_err(&unary_function(), CommandOptions::numerical(-2), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 2, actual: 0 },
    );
  }

  #[test]
  fn test_unary_function_command_with_arg_negative_two_on_stack_size_one() {
    let input_stack = vec![10];
    let error = act_on_stack_err(&unary_function(), CommandOptions::numerical(-2), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 2, actual: 1 },
    );
  }

  #[test]
  fn test_unary_function_command_with_arg_negative_two_on_stack_size_two() {
    let input_stack = vec![10, 20];
    let output_stack = act_on_stack(&unary_function(), CommandOptions::numerical(-2), input_stack);
    assert_eq!(
      output_stack,
      Stack::from(vec![
        Expr::call("test_func", vec![Expr::from(10)]),
        Expr::from(20),
      ]),
    );
  }

  #[test]
  fn test_binary_function_command() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&binary_function(), CommandOptions::default(), input_stack);
    assert_eq!(
      output_stack,
      Stack::from(vec![
        Expr::from(10),
        Expr::from(20),
        Expr::call("test_func", vec![Expr::from(30), Expr::from(40)]),
      ]),
    );
  }

  #[test]
  fn test_binary_function_command_with_keep_arg() {
    let opts = CommandOptions::default().with_keep_modifier();
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&binary_function(), opts, input_stack);
    assert_eq!(
      output_stack,
      Stack::from(vec![
        Expr::from(10),
        Expr::from(20),
        Expr::from(30),
        Expr::from(40),
        Expr::call("test_func", vec![Expr::from(30), Expr::from(40)]),
      ]),
    );
  }

  #[test]
  fn test_binary_function_command_on_stack_size_two() {
    let input_stack = vec![10, 20];
    let output_stack = act_on_stack(&binary_function(), CommandOptions::default(), input_stack);
    assert_eq!(
      output_stack,
      Stack::from(vec![
        Expr::call("test_func", vec![Expr::from(10), Expr::from(20)]),
      ]),
    );
  }

  #[test]
  fn test_binary_function_command_on_stack_size_one() {
    let input_stack = vec![10];
    let error = act_on_stack_err(&binary_function(), CommandOptions::default(), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 2, actual: 1 },
    );
  }

  #[test]
  fn test_binary_function_command_on_empty_stack() {
    let input_stack = vec![];
    let error = act_on_stack_err(&binary_function(), CommandOptions::default(), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 2, actual: 0 },
    );
  }

  #[test]
  fn test_binary_function_command_with_argument_two() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&binary_function(), CommandOptions::numerical(2), input_stack);
    assert_eq!(
      output_stack,
      Stack::from(vec![
        Expr::from(10),
        Expr::from(20),
        Expr::call("test_func", vec![Expr::from(30), Expr::from(40)]),
      ]),
    );
  }

  #[test]
  fn test_binary_function_command_with_argument_two_and_keep_arg() {
    let opts = CommandOptions::numerical(2).with_keep_modifier();
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&binary_function(), opts, input_stack);
    assert_eq!(
      output_stack,
      Stack::from(vec![
        Expr::from(10),
        Expr::from(20),
        Expr::from(30),
        Expr::from(40),
        Expr::call("test_func", vec![Expr::from(30), Expr::from(40)]),
      ]),
    );
  }

  #[test]
  fn test_binary_function_command_on_stack_size_two_with_arg_two() {
    let input_stack = vec![10, 20];
    let output_stack = act_on_stack(&binary_function(), CommandOptions::numerical(2), input_stack);
    assert_eq!(
      output_stack,
      Stack::from(vec![
        Expr::call("test_func", vec![Expr::from(10), Expr::from(20)]),
      ]),
    );
  }

  #[test]
  fn test_binary_function_command_on_stack_size_one_with_arg_two() {
    let input_stack = vec![10];
    let error = act_on_stack_err(&binary_function(), CommandOptions::numerical(2), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 2, actual: 1 },
    );
  }

  #[test]
  fn test_binary_function_command_on_empty_stack_with_arg_two() {
    let input_stack = vec![];
    let error = act_on_stack_err(&binary_function(), CommandOptions::numerical(2), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 2, actual: 0 },
    );
  }

  #[test]
  fn test_binary_function_command_with_argument_one() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&binary_function(), CommandOptions::numerical(1), input_stack);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40]));
  }

  #[test]
  fn test_binary_function_command_with_argument_one_and_keep_arg() {
    let opts = CommandOptions::numerical(1).with_keep_modifier();
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&binary_function(), opts, input_stack);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 40]));
  }

  #[test]
  fn test_binary_function_command_on_stack_size_one_with_arg_one() {
    let input_stack = vec![10];
    let output_stack = act_on_stack(&binary_function(), CommandOptions::numerical(1), input_stack);
    assert_eq!(output_stack, stack_of(vec![10]));
  }

  #[test]
  fn test_binary_function_command_on_empty_stack_with_arg_one() {
    let input_stack = vec![];
    let error = act_on_stack_err(&binary_function(), CommandOptions::numerical(1), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 1, actual: 0 },
    );
  }

  #[test]
  fn test_binary_function_command_with_positive_arg() {
    let input_stack = vec![10, 20, 30, 40, 50];
    let output_stack = act_on_stack(&binary_function(), CommandOptions::numerical(4), input_stack);

    fn test_func(a: Expr, b: Expr) -> Expr {
      Expr::call("test_func", vec![a, b])
    }

    assert_eq!(
      output_stack,
      Stack::from(vec![
        Expr::from(10),
        test_func(
          test_func(
            test_func(
              Expr::from(20),
              Expr::from(30),
            ),
            Expr::from(40),
          ),
          Expr::from(50),
        ),
      ]),
    );
  }

  #[test]
  fn test_binary_function_command_with_positive_arg_and_keep_arg() {
    let opts = CommandOptions::numerical(4).with_keep_modifier();
    let input_stack = vec![10, 20, 30, 40, 50];
    let output_stack = act_on_stack(&binary_function(), opts, input_stack);

    fn test_func(a: Expr, b: Expr) -> Expr {
      Expr::call("test_func", vec![a, b])
    }

    assert_eq!(
      output_stack,
      Stack::from(vec![
        Expr::from(10),
        Expr::from(20),
        Expr::from(30),
        Expr::from(40),
        Expr::from(50),
        test_func(
          test_func(
            test_func(
              Expr::from(20),
              Expr::from(30),
            ),
            Expr::from(40),
          ),
          Expr::from(50),
        ),
      ]),
    );
  }

  #[test]
  fn test_binary_function_command_with_positive_arg_equal_to_stack_size() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&binary_function(), CommandOptions::numerical(4), input_stack);

    fn test_func(a: Expr, b: Expr) -> Expr {
      Expr::call("test_func", vec![a, b])
    }

    assert_eq!(
      output_stack,
      Stack::from(vec![
        test_func(
          test_func(
            test_func(
              Expr::from(10),
              Expr::from(20),
            ),
            Expr::from(30),
          ),
          Expr::from(40),
        ),
      ]),
    );
  }

  #[test]
  fn test_binary_function_command_with_positive_arg_equal_to_stack_size_and_keep_arg() {
    let opts = CommandOptions::numerical(4).with_keep_modifier();
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&binary_function(), opts, input_stack);

    fn test_func(a: Expr, b: Expr) -> Expr {
      Expr::call("test_func", vec![a, b])
    }

    assert_eq!(
      output_stack,
      Stack::from(vec![
        Expr::from(10),
        Expr::from(20),
        Expr::from(30),
        Expr::from(40),
        test_func(
          test_func(
            test_func(
              Expr::from(10),
              Expr::from(20),
            ),
            Expr::from(30),
          ),
          Expr::from(40),
        ),
      ]),
    );
  }

  #[test]
  fn test_binary_function_command_with_positive_arg_and_stack_too_small() {
    let input_stack = vec![10, 20, 30];
    let error = act_on_stack_err(&binary_function(), CommandOptions::numerical(4), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 4, actual: 3 },
    )
  }

  #[test]
  fn test_binary_function_command_with_positive_arg_and_empty_stack() {
    let input_stack = vec![];
    let error = act_on_stack_err(&binary_function(), CommandOptions::numerical(3), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 3, actual: 0 },
    )
  }

  #[test]
  fn test_binary_function_command_with_arg_negative_one() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&binary_function(), CommandOptions::numerical(-1), input_stack);
    assert_eq!(
      output_stack,
      Stack::from(vec![
        Expr::from(10),
        Expr::from(20),
        Expr::call("test_func", vec![Expr::from(30), Expr::from(40)]),
      ]),
    );
  }

  #[test]
  fn test_binary_function_command_with_arg_negative_one_and_keep_arg() {
    let opts = CommandOptions::numerical(-1).with_keep_modifier();
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&binary_function(), opts, input_stack);
    assert_eq!(
      output_stack,
      Stack::from(vec![
        Expr::from(10),
        Expr::from(20),
        Expr::from(30),
        Expr::from(40),
        Expr::call("test_func", vec![Expr::from(30), Expr::from(40)]),
      ]),
    );
  }

  #[test]
  fn test_binary_function_command_with_arg_negative_one_on_stack_size_one() {
    let input_stack = vec![10];
    let error = act_on_stack_err(&binary_function(), CommandOptions::numerical(-1), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 2, actual: 1 },
    )
  }

  #[test]
  fn test_binary_function_command_with_negative_arg() {
    let input_stack = vec![10, 20, 30, 40, 50];
    let output_stack = act_on_stack(&binary_function(), CommandOptions::numerical(-3), input_stack);
    assert_eq!(
      output_stack,
      Stack::from(vec![
        Expr::from(10),
        Expr::call("test_func", vec![Expr::from(20), Expr::from(50)]),
        Expr::call("test_func", vec![Expr::from(30), Expr::from(50)]),
        Expr::call("test_func", vec![Expr::from(40), Expr::from(50)]),
      ]),
    );
  }

  #[test]
  fn test_binary_function_command_with_negative_arg_and_keep_arg() {
    let opts = CommandOptions::numerical(-3).with_keep_modifier();
    let input_stack = vec![10, 20, 30, 40, 50];
    let output_stack = act_on_stack(&binary_function(), opts, input_stack);
    assert_eq!(
      output_stack,
      Stack::from(vec![
        Expr::from(10),
        Expr::from(20),
        Expr::from(30),
        Expr::from(40),
        Expr::from(50),
        Expr::call("test_func", vec![Expr::from(20), Expr::from(50)]),
        Expr::call("test_func", vec![Expr::from(30), Expr::from(50)]),
        Expr::call("test_func", vec![Expr::from(40), Expr::from(50)]),
      ]),
    );
  }

  #[test]
  fn test_binary_function_command_with_negative_arg_and_too_small_stack() {
    let input_stack = vec![10, 20, 30, 40];
    let error = act_on_stack_err(&binary_function(), CommandOptions::numerical(-4), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 5, actual: 4 },
    )
  }

  #[test]
  fn test_binary_function_command_with_negative_arg_and_empty_stack() {
    let input_stack = vec![];
    let error = act_on_stack_err(&binary_function(), CommandOptions::numerical(-4), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 5, actual: 0 },
    )
  }
}
