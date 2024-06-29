
//! Commands which operate on composite data structures, such as
//! vectors, to create or destructure them.

use super::arguments::{NullaryArgumentSchema, validate_schema};
use super::base::{Command, CommandContext, CommandOutput};
use crate::util::prism::Prism;
use crate::errorlist::ErrorList;
use crate::expr::Expr;
use crate::expr::prisms;
use crate::expr::atom::Atom;
use crate::expr::vector::Vector;
use crate::expr::simplifier::error::DomainError;
use crate::state::ApplicationState;
use crate::stack::base::StackLike;
use crate::stack::keepable::KeepableStack;

use num::BigInt;

/// `PackCommand` packs several stack elements into a vector.
///
/// With no numerical argument, `PackCommand` pops a single value off
/// the stack (which must be an integer). That value is treated as the
/// numerical argument.
///
/// With a non-negative numerical argument N, `PackCommand` pops N
/// values off the stack and pushes a single vector containing those
/// values.
///
/// Negative numerical arguments are not currently supported but might
/// be supported in the future.
#[derive(Debug, Default)]
pub struct PackCommand {
  _priv: (),
}

/// `UnpackCommand` unpacks the top stack element into several stack
/// elements.
///
/// If given a function call on top of the stack, the arguments to the
/// call are pushed onto the stack. This includes the case of a
/// vector, which is represented as a call to the "vector" function.
///
/// Most atoms cannot be destructured, with the exception of complex
/// number atoms, which will be destructured into the real part and
/// the imaginary part.
///
/// This command respects the "keep" modifier but does not use the
/// numerical argument.
#[derive(Debug, Default)]
pub struct UnpackCommand {
  _priv: (),
}

/// `RepeatCommand` pops one stack element `expr` and pushes the
/// expression `repeat(expr, n)`, where `n` is the numerical argument
/// to the command. The numerical argument defaults to 2 if not
/// supplied.
#[derive(Debug, Default)]
pub struct RepeatCommand {
  _priv: (),
}

impl PackCommand {
  pub fn new() -> Self {
    Self::default()
  }

  fn pop_and_construct_vector(
    state: &mut ApplicationState,
    context: &CommandContext,
    arg: usize,
  ) -> anyhow::Result<Vector> {
    let mut stack = KeepableStack::new(state.main_stack_mut(), context.opts.keep_modifier);
    let values = stack.pop_several(arg)?;
    Ok(Vector::from(values))
  }

  /// Pops a non-negative integer from the stack. If the stack is
  /// empty, returns an error. If the top of the stack is not a
  /// positive integer or is too large to store in a Rust usize,
  /// returns an error and leaves the stack unmodified.
  fn pop_non_negative_integer(
    stack: &mut impl StackLike<Elem=Expr>,
  ) -> anyhow::Result<usize> {
    let elem = stack.pop()?;
    match prisms::expr_to_usize().narrow_type(elem) {
      Err(elem) => {
        // Failed to convert, so put it back.
        stack.push(elem.clone());
        Err(anyhow::anyhow!(DomainError::new(format!("Expected small positive integer, got {}", elem))))
      }
      Ok(arg) => {
        Ok(arg)
      }
    }
  }
}

impl UnpackCommand {
  pub fn new() -> Self {
    Self::default()
  }
}

impl RepeatCommand {
  pub fn new() -> Self {
    Self::default()
  }
}

impl Command for PackCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    context: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    validate_schema(&NullaryArgumentSchema::new(), args)?;
    state.undo_stack_mut().push_cut();

    let mut errors = ErrorList::new();
    if let Some(arg) = context.opts.argument {
      anyhow::ensure!(arg >= 0, "PackCommand: negative argument not supported, got {arg}");
      // Pop `arg` values and construct a vector.
      let vector = PackCommand::pop_and_construct_vector(state, context, arg as usize)?;
      let expr = context.simplify_expr(vector.into(), &mut errors);
      state.main_stack_mut().push(expr);
    } else {
      // Pop one value, use that to determine the length of the
      // vector. Take care to respect the "keep" modifier.
      let arg = PackCommand::pop_non_negative_integer(&mut state.main_stack_mut())?;
      // TODO (CLEANUP): Messy code pattern here to perform stack
      // cleanup in case of error.
      let vector = match PackCommand::pop_and_construct_vector(state, context, arg) {
        Ok(vector) => vector,
        Err(err) => {
          state.main_stack_mut().push(Expr::from(BigInt::from(arg)));
          return Err(err);
        }
      };
      let expr = context.simplify_expr(vector.into(), &mut errors);
      if context.opts.keep_modifier {
        state.main_stack_mut().push(Expr::from(BigInt::from(arg)));
      }
      state.main_stack_mut().push(expr);
    }

    Ok(CommandOutput::from_errors(errors))
  }
}

impl Command for UnpackCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    context: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    validate_schema(&NullaryArgumentSchema::new(), args)?;
    state.undo_stack_mut().push_cut();

    let mut errors = ErrorList::new();
    let mut stack = KeepableStack::new(state.main_stack_mut(), context.opts.keep_modifier);
    match stack.pop()? {
      Expr::Call(_, args) => {
        let args = args.into_iter().map(|arg| context.simplify_expr(arg, &mut errors));
        stack.push_several(args);
      }
      expr @ Expr::Atom(Atom::Number(_) | Atom::Var(_) | Atom::String(_)) => {
        if !context.opts.keep_modifier {
          // If we actually popped the value, then push it back since
          // this is an error condition.
          stack.push(expr.clone());
        }
        return Err(anyhow::anyhow!(DomainError::new(format!("Cannot unpack {expr}"))));
      }
    }
    Ok(CommandOutput::from_errors(errors))
  }
}

impl Command for RepeatCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    context: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    validate_schema(&NullaryArgumentSchema::new(), args)?;
    state.undo_stack_mut().push_cut();

    let arg = context.opts.argument.unwrap_or(2);
    let mut errors = ErrorList::new();
    let mut stack = KeepableStack::new(state.main_stack_mut(), context.opts.keep_modifier);

    let expr = stack.pop()?;
    let expr = Expr::call("repeat", vec![expr, Expr::from(arg)]);
    let expr = context.simplify_expr(expr, &mut errors);
    stack.push(expr);

    Ok(CommandOutput::from_errors(errors))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::stack::{Stack, StackError};
  use crate::command::test_utils::{act_on_stack, act_on_stack_err, act_on_stack_any_err};
  use crate::command::options::CommandOptions;
  use crate::expr::number::{Number, ComplexNumber};

  #[test]
  fn test_simple_pack_vector() {
    let opts = CommandOptions::numerical(2);
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&PackCommand::new(), opts, input_stack);
    assert_eq!(output_stack, Stack::from(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call(Vector::FUNCTION_NAME, vec![Expr::from(30), Expr::from(40)]),
    ]));
  }

  #[test]
  fn test_pack_vector_whole_stack() {
    let opts = CommandOptions::numerical(4);
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&PackCommand::new(), opts, input_stack);
    assert_eq!(output_stack, Stack::from(vec![
      Expr::call(Vector::FUNCTION_NAME, vec![
        Expr::from(10),
        Expr::from(20),
        Expr::from(30),
        Expr::from(40),
      ]),
    ]));
  }

  #[test]
  fn test_pack_vector_stack_too_small() {
    let opts = CommandOptions::numerical(4);
    let input_stack = vec![10, 20, 30];
    let err = act_on_stack_err(&PackCommand::new(), opts, input_stack);
    assert_eq!(err, StackError::NotEnoughElements { expected: 4, actual: 3 });
  }

  #[test]
  fn test_pack_vector_arg_zero() {
    let opts = CommandOptions::numerical(0);
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&PackCommand::new(), opts, input_stack);
    assert_eq!(output_stack, Stack::from(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::from(30),
      Expr::from(40),
      Expr::call(Vector::FUNCTION_NAME, vec![]),
    ]));
  }

  #[test]
  fn test_pack_vector_arg_one() {
    let opts = CommandOptions::numerical(1);
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&PackCommand::new(), opts, input_stack);
    assert_eq!(output_stack, Stack::from(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::from(30),
      Expr::call(Vector::FUNCTION_NAME, vec![Expr::from(40)]),
    ]));
  }

  #[test]
  fn test_pack_vector_with_keep_arg() {
    let opts = CommandOptions::numerical(2).with_keep_modifier();
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&PackCommand::new(), opts, input_stack);
    assert_eq!(output_stack, Stack::from(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::from(30),
      Expr::from(40),
      Expr::call(Vector::FUNCTION_NAME, vec![Expr::from(30), Expr::from(40)]),
    ]));
  }

  #[test]
  fn test_pack_vector_no_prefix_arg() {
    let opts = CommandOptions::default();
    let input_stack = vec![10, 20, 30, 40, 2];
    let output_stack = act_on_stack(&PackCommand::new(), opts, input_stack);
    assert_eq!(output_stack, Stack::from(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call(Vector::FUNCTION_NAME, vec![Expr::from(30), Expr::from(40)]),
    ]));
  }

  #[test]
  fn test_pack_vector_with_keep_arg_but_no_prefix_arg() {
    let opts = CommandOptions::default().with_keep_modifier();
    let input_stack = vec![10, 20, 30, 40, 2];
    let output_stack = act_on_stack(&PackCommand::new(), opts, input_stack);
    assert_eq!(output_stack, Stack::from(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::from(30),
      Expr::from(40),
      Expr::from(2),
      Expr::call(Vector::FUNCTION_NAME, vec![Expr::from(30), Expr::from(40)]),
    ]));
  }

  #[test]
  fn test_pack_vector_no_prefix_arg_and_empty_stack() {
    let opts = CommandOptions::default();
    let input_stack = Vec::<Expr>::new();
    let err = act_on_stack_err(&PackCommand::new(), opts, input_stack);
    assert_eq!(err, StackError::NotEnoughElements { expected: 1, actual: 0 });
  }

  #[test]
  fn test_pack_vector_no_prefix_arg_and_not_enough_arguments() {
    let opts = CommandOptions::default();
    let input_stack = vec![10, 20, 5];
    let err = act_on_stack_err(&PackCommand::new(), opts, input_stack);
    assert_eq!(err, StackError::NotEnoughElements { expected: 5, actual: 2 });
  }

  #[test]
  fn test_pack_vector_keep_arg_but_no_prefix_arg_and_not_enough_arguments() {
    let opts = CommandOptions::default().with_keep_modifier();
    let input_stack = vec![10, 20, 5];
    let err = act_on_stack_err(&PackCommand::new(), opts, input_stack);
    assert_eq!(err, StackError::NotEnoughElements { expected: 5, actual: 2 });
  }

  #[test]
  fn test_pack_vector_no_prefix_arg_negative_top_of_stack() {
    let opts = CommandOptions::default();
    let input_stack = vec![10, 20, -2];
    let err = act_on_stack_any_err(&PackCommand::new(), opts, input_stack);
    let err = err.downcast::<DomainError>().unwrap();
    assert_eq!(err.explanation, "Expected small positive integer, got -2");
  }

  #[test]
  fn test_pack_vector_no_prefix_arg_negative_top_of_stack_with_keep_arg() {
    let opts = CommandOptions::default().with_keep_modifier();
    let input_stack = vec![10, 20, -2];
    let err = act_on_stack_any_err(&PackCommand::new(), opts, input_stack);
    let err = err.downcast::<DomainError>().unwrap();
    assert_eq!(err.explanation, "Expected small positive integer, got -2");
  }

  #[test]
  fn test_pack_vector_no_prefix_arg_invalid_top_of_stack() {
    let opts = CommandOptions::default();
    let input_stack = vec![Expr::from(10), Expr::from(20), Expr::var("x").unwrap()];
    let err = act_on_stack_any_err(&PackCommand::new(), opts, input_stack);
    let err = err.downcast::<DomainError>().unwrap();
    assert_eq!(err.explanation, "Expected small positive integer, got x");
  }

  #[test]
  fn test_unpack_vector() {
    let opts = CommandOptions::default();
    let input_stack = vec![
      Expr::from(10),
      Expr::call(Vector::FUNCTION_NAME, vec![Expr::from(20), Expr::from(30), Expr::from(40)]),
    ];
    let output_stack = act_on_stack(&UnpackCommand::new(), opts, input_stack);
    assert_eq!(output_stack, Stack::from(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::from(30),
      Expr::from(40),
    ]));
  }

  #[test]
  fn test_unpack_vector_with_keep_arg() {
    let opts = CommandOptions::default().with_keep_modifier();
    let input_stack = vec![
      Expr::from(10),
      Expr::call(Vector::FUNCTION_NAME, vec![Expr::from(20), Expr::from(30), Expr::from(40)]),
    ];
    let output_stack = act_on_stack(&UnpackCommand::new(), opts, input_stack);
    assert_eq!(output_stack, Stack::from(vec![
      Expr::from(10),
      Expr::call(Vector::FUNCTION_NAME, vec![Expr::from(20), Expr::from(30), Expr::from(40)]),
      Expr::from(20),
      Expr::from(30),
      Expr::from(40),
    ]));
  }

  #[test]
  fn test_unpack_complex() {
    let opts = CommandOptions::default();
    let input_stack = vec![
      Expr::from(ComplexNumber::new(Number::from(1), Number::from(3))),
    ];
    let output_stack = act_on_stack(&UnpackCommand::new(), opts, input_stack);
    assert_eq!(output_stack, Stack::from(vec![
      Expr::from(1),
      Expr::from(3),
    ]));
  }

  #[test]
  fn test_unpack_arbitrary_call() {
    let opts = CommandOptions::default();
    let input_stack = vec![
      Expr::from(10),
      Expr::call("+", vec![Expr::from(20), Expr::from(30), Expr::from(40)]),
    ];
    let output_stack = act_on_stack(&UnpackCommand::new(), opts, input_stack);
    assert_eq!(output_stack, Stack::from(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::from(30),
      Expr::from(40),
    ]));
  }

  #[test]
  fn test_unpack_arbitrary_call_with_keep_arg() {
    let opts = CommandOptions::default().with_keep_modifier();
    let input_stack = vec![
      Expr::from(10),
      Expr::call("+", vec![Expr::from(20), Expr::from(30), Expr::from(40)]),
    ];
    let output_stack = act_on_stack(&UnpackCommand::new(), opts, input_stack);
    assert_eq!(output_stack, Stack::from(vec![
      Expr::from(10),
      Expr::call("+", vec![Expr::from(20), Expr::from(30), Expr::from(40)]),
      Expr::from(20),
      Expr::from(30),
      Expr::from(40),
    ]));
  }

  #[test]
  fn test_unpack_with_empty_stack() {
    let opts = CommandOptions::default();
    let input_stack = Vec::<Expr>::new();
    let err = act_on_stack_err(&UnpackCommand::new(), opts, input_stack);
    assert_eq!(err, StackError::NotEnoughElements { expected: 1, actual: 0 });
  }

  #[test]
  fn test_unpack_with_invalid_top_of_stack() {
    let opts = CommandOptions::default();
    let input_stack = vec![Expr::from(10), Expr::from(20), Expr::var("x").unwrap()];
    let err = act_on_stack_any_err(&UnpackCommand::new(), opts, input_stack);
    let err = err.downcast::<DomainError>().unwrap();
    assert_eq!(err.explanation, "Cannot unpack x");
  }

  #[test]
  fn test_unpack_with_invalid_top_of_stack_with_keep_arg() {
    let opts = CommandOptions::default().with_keep_modifier();
    let input_stack = vec![Expr::from(10), Expr::from(20), Expr::var("x").unwrap()];
    let err = act_on_stack_any_err(&UnpackCommand::new(), opts, input_stack);
    let err = err.downcast::<DomainError>().unwrap();
    assert_eq!(err.explanation, "Cannot unpack x");
  }

  #[test]
  fn test_repeat_with_no_arg() {
    let opts = CommandOptions::default();
    let input_stack = vec![Expr::from(10)];
    let output_stack = act_on_stack(&RepeatCommand::new(), opts, input_stack);
    assert_eq!(output_stack, Stack::from(vec![
      Expr::call("repeat", vec![Expr::from(10), Expr::from(2)]),
    ]));
  }

  #[test]
  fn test_repeat_with_arg() {
    let opts = CommandOptions::numerical(5);
    let input_stack = vec![Expr::from(10)];
    let output_stack = act_on_stack(&RepeatCommand::new(), opts, input_stack);
    assert_eq!(output_stack, Stack::from(vec![
      Expr::call("repeat", vec![Expr::from(10), Expr::from(5)]),
    ]));
  }

  #[test]
  fn test_repeat_with_arg_and_keep_arg() {
    let opts = CommandOptions::numerical(5).with_keep_modifier();
    let input_stack = vec![Expr::from(10)];
    let output_stack = act_on_stack(&RepeatCommand::new(), opts, input_stack);
    assert_eq!(output_stack, Stack::from(vec![
      Expr::from(10),
      Expr::call("repeat", vec![Expr::from(10), Expr::from(5)]),
    ]));
  }

  #[test]
  fn test_repeat_with_empty_stack() {
    let opts = CommandOptions::default();
    let input_stack = Vec::<Expr>::new();
    let err = act_on_stack_err(&RepeatCommand::new(), opts, input_stack);
    assert_eq!(err, StackError::NotEnoughElements { expected: 1, actual: 0 });
  }
}
