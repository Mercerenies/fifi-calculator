
use crate::expr::Expr;
use crate::expr::function::table::FunctionTable;
use crate::expr::function::flags::FunctionFlags;
use super::base::{Simplifier, SimplifierContext};

/// `InvolutionSimplifier` is a [`Simplifier`] that performs the
/// simplification described in [`FunctionFlags::IS_INVOLUTION`].
/// Specifically, if `f` is a function with that flag set, then this
/// simplifier will simplify applications of the form `f(f(x))` to
/// simply `x`.
///
/// This simplification only affects unary applications of the
/// function `f`. Functions without the `IS_INVOLUTION` flag set will
/// be left alone.
#[derive(Debug)]
pub struct InvolutionSimplifier<'a> {
  function_table: &'a FunctionTable,
}

impl<'a> InvolutionSimplifier<'a> {
  pub fn new(function_table: &'a FunctionTable) -> Self {
    Self { function_table }
  }
}

impl<'a> Simplifier for InvolutionSimplifier<'a> {
  fn simplify_expr_part(&self, expr: Expr, _ctx: &mut SimplifierContext) -> Expr {
    match expr {
      Expr::Call(function_name, mut args) => {
        let Some(known_function) = self.function_table.get(&function_name) else {
          return Expr::Call(function_name, args);
        };
        if known_function.flags().contains(FunctionFlags::IS_INVOLUTION) {
          if args.len() == 1 && args[0].head() == Some(&function_name) && args[0].arity() == 1 {
            let Expr::Call(_, mut inner_args) = args.remove(0) else { unreachable!() };
            return inner_args.remove(0);
          }
        }
        Expr::Call(function_name, args)
      }
      expr => {
        // Pass through
        expr
      }
    }
  }
}
