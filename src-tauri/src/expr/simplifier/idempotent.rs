
use crate::expr::Expr;
use crate::expr::function::table::FunctionTable;
use crate::expr::function::flags::FunctionFlags;
use super::base::{Simplifier, SimplifierContext};

/// `IdempotenceSimplifier` is a [`Simplifier`] that performs the
/// simplification described in [`FunctionFlags::IS_IDEMPOTENT`].
/// Specifically, if `f` is a function with that flag set, then this
/// simplifier will simplify applications of the form `f(f(x))` to
/// simply `f(x)`.
///
/// This simplification only affects unary applications of the
/// function `f`. Functions without the `IS_IDEMPOTENT` flag set will
/// be left alone.
#[derive(Debug)]
pub struct IdempotenceSimplifier<'a> {
  function_table: &'a FunctionTable,
}

impl<'a> IdempotenceSimplifier<'a> {
  pub fn new(function_table: &'a FunctionTable) -> Self {
    Self { function_table }
  }
}

impl<'a> Simplifier for IdempotenceSimplifier<'a> {
  fn simplify_expr_part(&self, expr: Expr, _ctx: &mut SimplifierContext) -> Expr {
    match expr {
      Expr::Call(function_name, mut args) => {
        let Some(known_function) = self.function_table.get(&function_name) else {
          return Expr::Call(function_name, args);
        };
        if known_function.flags().contains(FunctionFlags::IS_IDEMPOTENT) &&
          args.len() == 1 &&
          args[0].head() == Some(&function_name) &&
          args[0].arity() == 1 {
            args.remove(0)
          } else {
            Expr::Call(function_name, args)
          }
      }
      expr => {
        // Pass through
        expr
      }
    }
  }
}
