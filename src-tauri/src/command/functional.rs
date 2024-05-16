
//! Commands that push functions onto the stack, using zero or more
//! arguments from the existing stack.

use super::base::{Command, CommandContext, CommandOutput};
use crate::state::ApplicationState;
use crate::stack::shuffle;
use crate::error::Error;
use crate::expr::Expr;
use crate::errorlist::ErrorList;

#[derive(Clone, Debug)]
pub struct PushConstantCommand {
  expr: Expr,
}

#[derive(Clone, Debug)]
pub struct UnaryFunctionCommand {
  function_name: String,
}

// Note for the future: I'm keeping these separate for now (rather
// than having a unified FunctionCommand which has an `arity`
// argument) since binary functions will have special treatment of
// prefix arguments.
#[derive(Clone, Debug)]
pub struct BinaryFunctionCommand {
  function_name: String,
}

impl PushConstantCommand {
  pub fn new(expr: Expr) -> PushConstantCommand {
    PushConstantCommand { expr }
  }
}

impl UnaryFunctionCommand {
  pub fn new(function_name: impl Into<String>) -> UnaryFunctionCommand {
    UnaryFunctionCommand { function_name: function_name.into() }
  }

  fn wrap_expr(&self, arg: Expr) -> Expr {
    Expr::Call(self.function_name.clone(), vec![arg])
  }
}

impl BinaryFunctionCommand {
  pub fn new(function_name: impl Into<String>) -> BinaryFunctionCommand {
    BinaryFunctionCommand { function_name: function_name.into() }
  }
}

impl Command for PushConstantCommand {
  fn run_command(&self, state: &mut ApplicationState, ctx: &CommandContext) -> Result<CommandOutput, Error> {
    let arg = ctx.opts.argument.unwrap_or(1).max(0);
    let mut errors = ErrorList::new();
    for _ in 0..arg {
      state.main_stack.push(ctx.simplifier.simplify_expr(self.expr.clone(), &mut errors));
    }
    Ok(CommandOutput::from_errors(errors))
  }
}

impl Command for UnaryFunctionCommand {
  fn run_command(&self, state: &mut ApplicationState, ctx: &CommandContext) -> Result<CommandOutput, Error> {
    let mut errors = ErrorList::new();
    let arg = ctx.opts.argument.unwrap_or(1);
    if arg > 0 {
      // Apply to top N elements.
      let values = shuffle::pop_several(&mut state.main_stack, arg as usize)?;
      let values = values.into_iter().map(|e| {
        ctx.simplifier.simplify_expr(self.wrap_expr(e), &mut errors)
      });
      state.main_stack.push_several(values);
    } else if arg < 0 {
      // Apply to single element N down on the stack.
      let e = shuffle::get_mut(&mut state.main_stack, - arg - 1)?;
      e.mutate(|e| ctx.simplifier.simplify_expr(self.wrap_expr(e), &mut errors));
    } else {
      // Apply to all elements.
      for e in state.main_stack.iter_mut() {
        e.mutate(|e| ctx.simplifier.simplify_expr(self.wrap_expr(e), &mut errors));
      }
    }
    Ok(CommandOutput::from_errors(errors))
  }
}

impl Command for BinaryFunctionCommand {
  fn run_command(&self, state: &mut ApplicationState, ctx: &CommandContext) -> Result<CommandOutput, Error> {
    //let mut errors = ErrorList::new();
    //let arg = ctx.opts.argument.unwrap_or(2);
    // TODO Use arg
    let mut errors = ErrorList::new();
    let (a, b) = shuffle::pop_two(&mut state.main_stack)?;
    let binary_call = Expr::Call(self.function_name.clone(), vec![a, b]);
    state.main_stack.push(ctx.simplifier.simplify_expr(binary_call, &mut errors));
    Ok(CommandOutput::from_errors(errors))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::command::test_utils::{act_on_stack, act_on_stack_err};
  use crate::stack::test_utils::stack_of;
  use crate::stack::Stack;
  use crate::stack::error::StackError;
  use crate::expr::number::Number;

  fn push_constant_zero() -> PushConstantCommand {
    PushConstantCommand::new(Expr::from(Number::from(0)))
  }

  fn unary_function() -> UnaryFunctionCommand {
    UnaryFunctionCommand::new("test_func")
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

}
