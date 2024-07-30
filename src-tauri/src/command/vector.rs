
//! Commands which operate on composite data structures, such as
//! vectors, to create or destructure them.

use super::arguments::{NullaryArgumentSchema, UnaryArgumentSchema, validate_schema};
use super::base::{Command, CommandContext, CommandOutput};
use crate::util;
use crate::util::prism::Prism;
use crate::errorlist::ErrorList;
use crate::expr::Expr;
use crate::expr::prisms;
use crate::expr::atom::Atom;
use crate::expr::number::{Number, ComplexNumber};
use crate::expr::vector::Vector;
use crate::expr::vector::matrix::Matrix;
use crate::expr::simplifier::error::DomainError;
use crate::expr::incomplete::{IncompleteObject, ObjectType, pop_until_delimiter};
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

/// `DiagonalCommand` constructs a diagonal matrix, using the top
/// stack element as its diagonal elements. An optional numerical
/// argument specifies the width and height of the matrix.
///
/// If the top stack element is a vector, then its length must match
/// the numerical argument (if given), and its elements will be used
/// as the diagonal elements.
///
/// If the top stack element is NOT a vector, then the numerical
/// argument is required, and the top stack element will be repeated
/// across the diagonal.
#[derive(Debug, Default)]
pub struct DiagonalCommand {
  _priv: (),
}

/// `IdentityMatrixCommand` constructs the identity matrix and pushes
/// it onto the stack. Expects a single nonnegative integer argument
/// specifying both the width and height of the resulting matrix.
#[derive(Debug, Default)]
pub struct IdentityMatrixCommand {
  _priv: (),
}

/// An `IndexedVectorCommand` expects a single integer argument. It
/// pops one element off the stack, which should generally be a
/// vector, and pushes a call to `self.function_name` with two
/// arguments: the popped element and the integer argument.
///
/// Respects the "keep" modifier.
#[derive(Debug)]
pub struct IndexedVectorCommand {
  function_name: String,
}

/// `VectorFromIncompleteObjectCommand` pops stack elements until it finds
/// the incomplete object [`ObjectType::LeftBracket`]. Then it pushes
/// a vector containing every value popped up to that point.
///
/// If we don't find the incomplete object or if we find the wrong
/// incomplete object, produces an error and does NOT modify the
/// stack.
///
/// Respects the "keep" modifier but does not use a numerical
/// argument.
#[derive(Debug, Default)]
pub struct VectorFromIncompleteObjectCommand {
  _priv: (),
}

/// `ComplexFromIncompleteObjectCommand` pops stack elements until it
/// finds the incomplete object [`ObjectType::LeftParen`]. Its
/// behavior from there depends on how many elements were popped.
///
/// * If one element was popped, that element is pushed back onto the
/// stack.
///
/// * If two elements were popped, they are treated as the real and
/// imaginary parts of a new complex number, which is pushed onto the
/// stack.
///
/// * If any other number of elements is popped, an error is produced.
///
/// In any error case (including lack of incomplete object, or a wrong
/// number of elements), the resulting stack is left in the same state
/// as before the command was run.
///
/// Respects the "keep" modifier but does not use a numerical
/// argument.
#[derive(Debug, Default)]
pub struct ComplexFromIncompleteObjectCommand {
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

impl DiagonalCommand {
  pub fn new() -> Self {
    Self::default()
  }
}

impl IdentityMatrixCommand {
  pub fn new() -> Self {
    Self::default()
  }

  fn argument_schema() -> UnaryArgumentSchema<prisms::StringToUsize, prisms::ParsedUsize> {
    UnaryArgumentSchema::new(
      "nonnegative integer".to_string(),
      prisms::StringToUsize,
    )
  }
}

impl IndexedVectorCommand {
  pub fn for_function(name: impl Into<String>) -> Self {
    Self { function_name: name.into() }
  }

  fn argument_schema() -> UnaryArgumentSchema<prisms::StringToI64, prisms::ParsedI64> {
    UnaryArgumentSchema::new(
      "integer".to_string(),
      prisms::StringToI64,
    )
  }
}

impl VectorFromIncompleteObjectCommand {
  pub fn new() -> Self {
    Self::default()
  }
}

impl ComplexFromIncompleteObjectCommand {
  pub fn new() -> Self {
    Self::default()
  }
}

pub fn nth_element_command() -> IndexedVectorCommand {
  IndexedVectorCommand::for_function("nth")
}

pub fn remove_nth_element_command() -> IndexedVectorCommand {
  IndexedVectorCommand::for_function("remove_nth")
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

    let calculation_mode = state.calculation_mode().clone();

    let mut errors = ErrorList::new();
    if let Some(arg) = context.opts.argument {
      anyhow::ensure!(arg >= 0, "PackCommand: negative argument not supported, got {arg}");
      // Pop `arg` values and construct a vector.
      let vector = PackCommand::pop_and_construct_vector(state, context, arg as usize)?;
      let expr = context.simplify_expr(vector.into(), calculation_mode, &mut errors);
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
      let expr = context.simplify_expr(vector.into(), calculation_mode, &mut errors);
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

    let calculation_mode = state.calculation_mode().clone();

    let mut errors = ErrorList::new();
    let mut stack = KeepableStack::new(state.main_stack_mut(), context.opts.keep_modifier);
    match stack.pop()? {
      Expr::Call(_, args) => {
        let args = args.into_iter().map(|arg| context.simplify_expr(arg, calculation_mode.clone(), &mut errors));
        stack.push_several(args);
      }
      Expr::Atom(Atom::String(s)) => {
        let chars = s.chars().map(|c| Expr::from(Number::from(c as usize)));
        stack.push_several(chars);
      }
      expr @ Expr::Atom(Atom::Number(_) | Atom::Var(_)) => {
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

    let calculation_mode = state.calculation_mode().clone();

    let arg = context.opts.argument.unwrap_or(2);
    let mut errors = ErrorList::new();
    let mut stack = KeepableStack::new(state.main_stack_mut(), context.opts.keep_modifier);

    let expr = stack.pop()?;
    let expr = Expr::call("repeat", vec![expr, Expr::from(arg)]);
    let expr = context.simplify_expr(expr, calculation_mode, &mut errors);
    stack.push(expr);

    Ok(CommandOutput::from_errors(errors))
  }
}

impl Command for DiagonalCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    context: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    validate_schema(&NullaryArgumentSchema::new(), args)?;
    state.undo_stack_mut().push_cut();

    let calculation_mode = state.calculation_mode().clone();
    let arg = context.opts.argument;
    let mut errors = ErrorList::new();
    let mut stack = KeepableStack::new(state.main_stack_mut(), context.opts.keep_modifier);

    let elem = stack.pop()?;
    let elems_vector = match prisms::ExprToVector.narrow_type(elem) {
      Err(scalar) => {
        // Scalar was provided; argument is not optional.
        let Some(arg) = arg else {
          if !context.opts.keep_modifier {
            stack.push(scalar);
          }
          anyhow::bail!("Missing numerical argument for diagonal matrix");
        };
        util::repeated(scalar, arg.max(0) as usize)
      }
      Ok(vector) => {
        if arg.is_some() && Some(vector.len()  as i64) != arg {
          if !context.opts.keep_modifier {
            stack.push(vector.into());
          }
          anyhow::bail!("Vector length mismatch");
        }
        Vec::from(vector)
      }
    };
    let expr = Expr::from(Matrix::diagonal(elems_vector));
    let expr = context.simplify_expr(expr, calculation_mode, &mut errors);
    stack.push(expr);
    Ok(CommandOutput::from_errors(errors))
  }
}

impl Command for IdentityMatrixCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    context: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    let matrix_dim = validate_schema(&Self::argument_schema(), args)?;
    let matrix_dim = usize::from(matrix_dim);
    state.undo_stack_mut().push_cut();

    let calculation_mode = state.calculation_mode().clone();
    let mut errors = ErrorList::new();
    let mut stack = KeepableStack::new(state.main_stack_mut(), context.opts.keep_modifier);

    let elems: Vec<_> = util::repeated(Expr::one(), matrix_dim);
    let identity_matrix = Expr::from(Matrix::diagonal(elems));
    let identity_matrix = context.simplify_expr(identity_matrix, calculation_mode, &mut errors);
    stack.push(identity_matrix);
    Ok(CommandOutput::from_errors(errors))
  }
}

impl Command for IndexedVectorCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    context: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    let index = validate_schema(&Self::argument_schema(), args)?;
    let index = i64::from(index);
    state.undo_stack_mut().push_cut();

    let calculation_mode = state.calculation_mode().clone();
    let mut errors = ErrorList::new();
    let mut stack = KeepableStack::new(state.main_stack_mut(), context.opts.keep_modifier);

    let vec = stack.pop()?;
    let expr = Expr::call(&self.function_name, vec![vec, Expr::from(index)]);
    let expr = context.simplify_expr(expr, calculation_mode, &mut errors);
    stack.push(expr);
    Ok(CommandOutput::from_errors(errors))
  }
}

impl Command for VectorFromIncompleteObjectCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    context: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    validate_schema(&NullaryArgumentSchema::new(), args)?;
    state.undo_stack_mut().push_cut();

    let mut stack = KeepableStack::new(state.main_stack_mut(), context.opts.keep_modifier);
    let elems = pop_until_delimiter(&mut stack, &IncompleteObject::new(ObjectType::LeftBracket))?;
    let vector = Vector::from(elems);
    stack.push(vector.into());
    Ok(CommandOutput::success())
  }
}

impl Command for ComplexFromIncompleteObjectCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    context: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    validate_schema(&NullaryArgumentSchema::new(), args)?;
    state.undo_stack_mut().push_cut();

    let calculation_mode = state.calculation_mode().clone();

    let mut stack = KeepableStack::new(state.main_stack_mut(), context.opts.keep_modifier);
    let elems = pop_until_delimiter(&mut stack, &IncompleteObject::new(ObjectType::LeftParen))?;
    match elems.len() {
      1 => {
        // Single element; push as-is.
        let [elem] = elems.try_into().unwrap();
        stack.push(elem);
        Ok(CommandOutput::success())
      }
      2 => {
        // Complex number.
        let mut errors = ErrorList::new();
        let [real, imag] = elems.try_into().unwrap();
        let complex = Expr::call("+", vec![
          real,
          Expr::call("*", vec![imag, Expr::from(ComplexNumber::ii())]),
        ]);
        let complex = context.simplify_expr(complex, calculation_mode, &mut errors);
        stack.push(complex);
        Ok(CommandOutput::from_errors(errors))
      }
      len => {
        if !context.opts.keep_modifier {
          // Return the stack elements if we didn't keep them.
          stack.push(IncompleteObject::new(ObjectType::LeftParen).into());
          stack.push_several(elems);
        }
        anyhow::bail!("Expected 1 or 2 elements, got {len}");
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::stack::{Stack, StackError};
  use crate::command::test_utils::act_on_stack;
  use crate::command::options::CommandOptions;
  use crate::expr::number::ComplexNumber;

  #[test]
  fn test_simple_pack_vector() {
    let opts = CommandOptions::numerical(2);
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&PackCommand::new(), opts, input_stack).unwrap();
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
    let output_stack = act_on_stack(&PackCommand::new(), opts, input_stack).unwrap();
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
    let err = act_on_stack(&PackCommand::new(), opts, input_stack).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(err, StackError::NotEnoughElements { expected: 4, actual: 3 });
  }

  #[test]
  fn test_pack_vector_arg_zero() {
    let opts = CommandOptions::numerical(0);
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&PackCommand::new(), opts, input_stack).unwrap();
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
    let output_stack = act_on_stack(&PackCommand::new(), opts, input_stack).unwrap();
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
    let output_stack = act_on_stack(&PackCommand::new(), opts, input_stack).unwrap();
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
    let output_stack = act_on_stack(&PackCommand::new(), opts, input_stack).unwrap();
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
    let output_stack = act_on_stack(&PackCommand::new(), opts, input_stack).unwrap();
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
    let err = act_on_stack(&PackCommand::new(), opts, input_stack).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(err, StackError::NotEnoughElements { expected: 1, actual: 0 });
  }

  #[test]
  fn test_pack_vector_no_prefix_arg_and_not_enough_arguments() {
    let opts = CommandOptions::default();
    let input_stack = vec![10, 20, 5];
    let err = act_on_stack(&PackCommand::new(), opts, input_stack).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(err, StackError::NotEnoughElements { expected: 5, actual: 2 });
  }

  #[test]
  fn test_pack_vector_keep_arg_but_no_prefix_arg_and_not_enough_arguments() {
    let opts = CommandOptions::default().with_keep_modifier();
    let input_stack = vec![10, 20, 5];
    let err = act_on_stack(&PackCommand::new(), opts, input_stack).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(err, StackError::NotEnoughElements { expected: 5, actual: 2 });
  }

  #[test]
  fn test_pack_vector_no_prefix_arg_negative_top_of_stack() {
    let opts = CommandOptions::default();
    let input_stack = vec![10, 20, -2];
    let err = act_on_stack(&PackCommand::new(), opts, input_stack).unwrap_err();
    let err = err.downcast::<DomainError>().unwrap();
    assert_eq!(err.explanation, "Expected small positive integer, got -2");
  }

  #[test]
  fn test_pack_vector_no_prefix_arg_negative_top_of_stack_with_keep_arg() {
    let opts = CommandOptions::default().with_keep_modifier();
    let input_stack = vec![10, 20, -2];
    let err = act_on_stack(&PackCommand::new(), opts, input_stack).unwrap_err();
    let err = err.downcast::<DomainError>().unwrap();
    assert_eq!(err.explanation, "Expected small positive integer, got -2");
  }

  #[test]
  fn test_pack_vector_no_prefix_arg_invalid_top_of_stack() {
    let opts = CommandOptions::default();
    let input_stack = vec![Expr::from(10), Expr::from(20), Expr::var("x").unwrap()];
    let err = act_on_stack(&PackCommand::new(), opts, input_stack).unwrap_err();
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
    let output_stack = act_on_stack(&UnpackCommand::new(), opts, input_stack).unwrap();
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
    let output_stack = act_on_stack(&UnpackCommand::new(), opts, input_stack).unwrap();
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
      Expr::from(ComplexNumber::new(1, 3)),
    ];
    let output_stack = act_on_stack(&UnpackCommand::new(), opts, input_stack).unwrap();
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
    let output_stack = act_on_stack(&UnpackCommand::new(), opts, input_stack).unwrap();
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
    let output_stack = act_on_stack(&UnpackCommand::new(), opts, input_stack).unwrap();
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
    let err = act_on_stack(&UnpackCommand::new(), opts, input_stack).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(err, StackError::NotEnoughElements { expected: 1, actual: 0 });
  }

  #[test]
  fn test_unpack_with_invalid_top_of_stack() {
    let opts = CommandOptions::default();
    let input_stack = vec![Expr::from(10), Expr::from(20), Expr::var("x").unwrap()];
    let err = act_on_stack(&UnpackCommand::new(), opts, input_stack).unwrap_err();
    let err = err.downcast::<DomainError>().unwrap();
    assert_eq!(err.explanation, "Cannot unpack x");
  }

  #[test]
  fn test_unpack_with_invalid_top_of_stack_with_keep_arg() {
    let opts = CommandOptions::default().with_keep_modifier();
    let input_stack = vec![Expr::from(10), Expr::from(20), Expr::var("x").unwrap()];
    let err = act_on_stack(&UnpackCommand::new(), opts, input_stack).unwrap_err();
    let err = err.downcast::<DomainError>().unwrap();
    assert_eq!(err.explanation, "Cannot unpack x");
  }

  #[test]
  fn test_repeat_with_no_arg() {
    let opts = CommandOptions::default();
    let input_stack = vec![Expr::from(10)];
    let output_stack = act_on_stack(&RepeatCommand::new(), opts, input_stack).unwrap();
    assert_eq!(output_stack, Stack::from(vec![
      Expr::call("repeat", vec![Expr::from(10), Expr::from(2)]),
    ]));
  }

  #[test]
  fn test_repeat_with_arg() {
    let opts = CommandOptions::numerical(5);
    let input_stack = vec![Expr::from(10)];
    let output_stack = act_on_stack(&RepeatCommand::new(), opts, input_stack).unwrap();
    assert_eq!(output_stack, Stack::from(vec![
      Expr::call("repeat", vec![Expr::from(10), Expr::from(5)]),
    ]));
  }

  #[test]
  fn test_repeat_with_arg_and_keep_arg() {
    let opts = CommandOptions::numerical(5).with_keep_modifier();
    let input_stack = vec![Expr::from(10)];
    let output_stack = act_on_stack(&RepeatCommand::new(), opts, input_stack).unwrap();
    assert_eq!(output_stack, Stack::from(vec![
      Expr::from(10),
      Expr::call("repeat", vec![Expr::from(10), Expr::from(5)]),
    ]));
  }

  #[test]
  fn test_repeat_with_empty_stack() {
    let opts = CommandOptions::default();
    let input_stack = Vec::<Expr>::new();
    let err = act_on_stack(&RepeatCommand::new(), opts, input_stack).unwrap_err();
    let err = err.downcast::<StackError>().unwrap();
    assert_eq!(err, StackError::NotEnoughElements { expected: 1, actual: 0 });
  }
}
