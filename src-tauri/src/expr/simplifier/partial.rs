
//! Simplifiers for partial evaluation of an expression which might
//! still contain unknowns.

use crate::expr::Expr;
use crate::expr::function::Function;
use crate::expr::function::table::FunctionTable;
use super::base::{Simplifier, SimplifierContext};

/// This simplifier removes any known identity values from
/// expressions. For instance, the values 0, 0.0, and (0, 0) will be
/// removed from addition operations, as zero is known to not affect
/// identity.
///
/// This simplifier uses the [`Function::is_identity`] function to
/// determine identity values.
#[derive(Debug)]
pub struct IdentityRemover<'a> {
  function_table: &'a FunctionTable,
}

impl<'a> IdentityRemover<'a> {
  pub fn new(function_table: &'a FunctionTable) -> Self {
    Self { function_table }
  }

  fn remove_identity_values(&self, function: &Function, function_name: String, mut args: Vec<Expr>) -> Expr {
    args.retain(|arg| !function.is_identity(arg));
    Expr::Call(function_name, args)
  }
}

impl Simplifier for IdentityRemover<'_> {
  fn simplify_expr_part(&self, expr: Expr, _ctx: &mut SimplifierContext) -> Expr {
    match expr {
      Expr::Call(function_name, args) => {
        let Some(known_function) = self.function_table.get(&function_name) else {
          return Expr::Call(function_name, args);
        };
        self.remove_identity_values(known_function, function_name, args)
      }
      expr => {
        // Pass through
        expr
      }
    }
  }
}
