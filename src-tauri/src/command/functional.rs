
//! Commands that push functions onto the stack, using zero or more
//! arguments from the existing stack.

use super::base::{Command, CommandContext, CommandOutput};
use crate::state::ApplicationState;
use crate::error::Error;
use crate::expr::Expr;
use crate::errorlist::ErrorList;

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
  function: Box<dyn Fn(Expr) -> Expr + Send + Sync>,
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
    UnaryFunctionCommand { function: Box::new(function) }
  }

  pub fn named(function_name: impl Into<String>) -> UnaryFunctionCommand {
    let function_name = function_name.into();
    UnaryFunctionCommand::new(move |arg| {
      Expr::Call(function_name.clone(), vec![arg])
    })
  }

  fn wrap_expr(&self, arg: Expr) -> Expr {
    (self.function)(arg)
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
  fn run_command(&self, state: &mut ApplicationState, ctx: &CommandContext) -> Result<CommandOutput, Error> {
    // TODO Undo/redo
    let arg = ctx.opts.argument.unwrap_or(1).max(0);
    let mut errors = ErrorList::new();
    for _ in 0..arg {
      state.main_stack_mut().push(ctx.simplifier.simplify_expr(self.expr.clone(), &mut errors));
    }
    Ok(CommandOutput::from_errors(errors))
  }
}

impl Command for UnaryFunctionCommand {
  fn run_command(&self, state: &mut ApplicationState, ctx: &CommandContext) -> Result<CommandOutput, Error> {
    // TODO Undo/redo
    let mut errors = ErrorList::new();
    let arg = ctx.opts.argument.unwrap_or(1);
    match arg.cmp(&0) {
      Ordering::Greater => {
        // Apply to top N elements.
        let values = state.main_stack_mut().pop_several(arg as usize)?;
        let values = values.into_iter().map(|e| {
          ctx.simplifier.simplify_expr(self.wrap_expr(e), &mut errors)
        });
        state.main_stack_mut().push_several(values);
      }
      Ordering::Less => {
        // Apply to single element N down on the stack.
        let e = state.main_stack_mut().get_mut(- arg - 1)?;
        e.mutate(|e| ctx.simplifier.simplify_expr(self.wrap_expr(e), &mut errors));
      }
      Ordering::Equal => {
        // Apply to all elements.
        for e in state.main_stack_mut().iter_mut() {
          e.mutate(|e| ctx.simplifier.simplify_expr(self.wrap_expr(e), &mut errors));
        }
      }
    }
    Ok(CommandOutput::from_errors(errors))
  }
}

impl Command for BinaryFunctionCommand {
  fn run_command(&self, state: &mut ApplicationState, ctx: &CommandContext) -> Result<CommandOutput, Error> {
    // TODO Undo/redo
    let mut errors = ErrorList::new();
    let arg = ctx.opts.argument.unwrap_or(2);
    match arg.cmp(&0) {
      Ordering::Greater => {
        // Perform reduction on top N elements.
        let values = state.main_stack_mut().pop_several(arg as usize)?;
        let result = values.into_iter().reduce(|a, b| {
          self.wrap_exprs(a, b)
        }).expect("Empty stack"); // expect safety: We popped at least one element off the stack
        let result = ctx.simplifier.simplify_expr(result, &mut errors);
        state.main_stack_mut().push(result);
      }
      Ordering::Less => {
        // Apply top to next N elements.
        let mut values = state.main_stack_mut().pop_several((- arg + 1) as usize)?;
        // expect safety: We popped at least two values, so removing one is safe.
        let second_argument = values.pop().expect("Empty stack");
        for e in values.iter_mut() {
          e.mutate(|e| ctx.simplifier.simplify_expr(self.wrap_exprs(e, second_argument.clone()), &mut errors));
        }
        state.main_stack_mut().push_several(values);
      }
      Ordering::Equal => {
        // Reduce entire stack.
        state.main_stack().check_stack_size(1)?;
        let values = state.main_stack_mut().pop_all();
        let result = values.into_iter().reduce(|a, b| {
          self.wrap_exprs(a, b)
        }).expect("Empty stack"); // expect safety: We just checked that the stack was non-empty
        let result = ctx.simplifier.simplify_expr(result, &mut errors);
        state.main_stack_mut().push(result);
      }
    }
    Ok(CommandOutput::from_errors(errors))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::command::test_utils::{act_on_stack, act_on_stack_err};
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
    let output_stack = act_on_stack(&push_constant_zero(), None, input_stack);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 0]));
  }

  #[test]
  fn test_push_constant_with_empty_stack() {
    let input_stack = vec![];
    let output_stack = act_on_stack(&push_constant_zero(), None, input_stack);
    assert_eq!(output_stack, stack_of(vec![0]));
  }

  #[test]
  fn test_push_constant_with_explicit_argument_one() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&push_constant_zero(), Some(1), input_stack);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 0]));
  }

  #[test]
  fn test_push_constant_with_positive_argument() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&push_constant_zero(), Some(3), input_stack);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40, 0, 0, 0]));
  }

  #[test]
  fn test_push_constant_with_negative_argument() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&push_constant_zero(), Some(-4), input_stack);
    // Does not change the stack
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40]));
  }

  #[test]
  fn test_push_constant_with_argument_of_zero() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&push_constant_zero(), Some(0), input_stack);
    // Does not change the stack
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40]));
  }

  #[test]
  fn test_unary_function_command() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&unary_function(), None, input_stack);
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
  fn test_unary_function_command_on_empty_stack() {
    let input_stack = vec![];
    let error = act_on_stack_err(&unary_function(), None, input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 1, actual: 0 },
    );
  }

  #[test]
  fn test_unary_function_command_with_explicit_arg_one() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&unary_function(), Some(1), input_stack);
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
    let error = act_on_stack_err(&unary_function(), Some(1), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 1, actual: 0 },
    );
  }

  #[test]
  fn test_unary_function_command_with_arg_two() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&unary_function(), Some(2), input_stack);
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
  fn test_unary_function_command_with_arg_two_on_empty_stack() {
    let input_stack = vec![];
    let error = act_on_stack_err(&unary_function(), Some(2), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 2, actual: 0 },
    );
  }

  #[test]
  fn test_unary_function_command_with_arg_two_on_stack_size_one() {
    let input_stack = vec![10];
    let error = act_on_stack_err(&unary_function(), Some(2), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 2, actual: 1 },
    );
  }

  #[test]
  fn test_unary_function_command_with_arg_zero() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&unary_function(), Some(0), input_stack);
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
  fn test_unary_function_command_with_arg_zero_on_empty_stack() {
    let input_stack = vec![];
    let output_stack = act_on_stack(&unary_function(), Some(0), input_stack);
    assert_eq!(output_stack, stack_of(vec![]));
  }

  #[test]
  fn test_unary_function_command_with_arg_negative_one() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&unary_function(), Some(-1), input_stack);
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
  fn test_unary_function_command_with_arg_negative_one_on_empty_stack() {
    let input_stack = vec![];
    let error = act_on_stack_err(&unary_function(), Some(1), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 1, actual: 0 },
    );
  }

  #[test]
  fn test_unary_function_command_with_arg_negative_two() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&unary_function(), Some(-2), input_stack);
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
  fn test_unary_function_command_with_arg_negative_two_on_empty_stack() {
    let input_stack = vec![];
    let error = act_on_stack_err(&unary_function(), Some(-2), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 2, actual: 0 },
    );
  }

  #[test]
  fn test_unary_function_command_with_arg_negative_two_on_stack_size_one() {
    let input_stack = vec![10];
    let error = act_on_stack_err(&unary_function(), Some(-2), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 2, actual: 1 },
    );
  }

  #[test]
  fn test_unary_function_command_with_arg_negative_two_on_stack_size_two() {
    let input_stack = vec![10, 20];
    let output_stack = act_on_stack(&unary_function(), Some(-2), input_stack);
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
    let output_stack = act_on_stack(&binary_function(), None, input_stack);
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
  fn test_binary_function_command_on_stack_size_two() {
    let input_stack = vec![10, 20];
    let output_stack = act_on_stack(&binary_function(), None, input_stack);
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
    let error = act_on_stack_err(&binary_function(), None, input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 2, actual: 1 },
    );
  }

  #[test]
  fn test_binary_function_command_on_empty_stack() {
    let input_stack = vec![];
    let error = act_on_stack_err(&binary_function(), None, input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 2, actual: 0 },
    );
  }

  #[test]
  fn test_binary_function_command_with_argument_two() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&binary_function(), Some(2), input_stack);
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
  fn test_binary_function_command_on_stack_size_two_with_arg_two() {
    let input_stack = vec![10, 20];
    let output_stack = act_on_stack(&binary_function(), Some(2), input_stack);
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
    let error = act_on_stack_err(&binary_function(), Some(2), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 2, actual: 1 },
    );
  }

  #[test]
  fn test_binary_function_command_on_empty_stack_with_arg_two() {
    let input_stack = vec![];
    let error = act_on_stack_err(&binary_function(), Some(2), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 2, actual: 0 },
    );
  }

  #[test]
  fn test_binary_function_command_with_argument_one() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&binary_function(), Some(1), input_stack);
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 40]));
  }

  #[test]
  fn test_binary_function_command_on_stack_size_one_with_arg_one() {
    let input_stack = vec![10];
    let output_stack = act_on_stack(&binary_function(), Some(1), input_stack);
    assert_eq!(output_stack, stack_of(vec![10]));
  }

  #[test]
  fn test_binary_function_command_on_empty_stack_with_arg_one() {
    let input_stack = vec![];
    let error = act_on_stack_err(&binary_function(), Some(1), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 1, actual: 0 },
    );
  }

  #[test]
  fn test_binary_function_command_with_positive_arg() {
    let input_stack = vec![10, 20, 30, 40, 50];
    let output_stack = act_on_stack(&binary_function(), Some(4), input_stack);

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
  fn test_binary_function_command_with_positive_arg_equal_to_stack_size() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&binary_function(), Some(4), input_stack);

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
  fn test_binary_function_command_with_positive_arg_and_stack_too_small() {
    let input_stack = vec![10, 20, 30];
    let error = act_on_stack_err(&binary_function(), Some(4), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 4, actual: 3 },
    )
  }

  #[test]
  fn test_binary_function_command_with_positive_arg_and_empty_stack() {
    let input_stack = vec![];
    let error = act_on_stack_err(&binary_function(), Some(3), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 3, actual: 0 },
    )
  }

  #[test]
  fn test_binary_function_command_with_arg_negative_one() {
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&binary_function(), Some(-1), input_stack);
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
  fn test_binary_function_command_with_arg_negative_one_on_stack_size_one() {
    let input_stack = vec![10];
    let error = act_on_stack_err(&binary_function(), Some(-1), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 2, actual: 1 },
    )
  }

  #[test]
  fn test_binary_function_command_with_negative_arg() {
    let input_stack = vec![10, 20, 30, 40, 50];
    let output_stack = act_on_stack(&binary_function(), Some(-3), input_stack);
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
  fn test_binary_function_command_with_negative_arg_and_too_small_stack() {
    let input_stack = vec![10, 20, 30, 40];
    let error = act_on_stack_err(&binary_function(), Some(-4), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 5, actual: 4 },
    )
  }

  #[test]
  fn test_binary_function_command_with_negative_arg_and_empty_stack() {
    let input_stack = vec![];
    let error = act_on_stack_err(&binary_function(), Some(-4), input_stack);
    assert_eq!(
      error,
      StackError::NotEnoughElements { expected: 5, actual: 0 },
    )
  }
}
