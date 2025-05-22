
use crate::expr::Expr;
use crate::expr::function::table::FunctionTable;
use crate::expr::function::flags::FunctionFlags;
use super::base::{Simplifier, SimplifierContext};

/// `FunctionFlattener` is a [`Simplifier`] that performs the
/// simplification described in [`FunctionFlags::PERMITS_FLATTENING`].
/// Specifically, if `f` is a function with that flag set, then this
/// simplifier will flatten nested applications of the function `f`.
///
/// Any functions which do not have the `PERMITS_FLATTENING` flag set,
/// including functions which are completely unknown to the engine,
/// will be left alone.
///
/// Examples:
///
/// ```text
/// f(x, f(y, z), t) ==> f(x, y, z, t)
/// f(f(), a, b) ==> f(a, b)
/// f(f(f(x))) ==> f(x)
/// ```
#[derive(Debug)]
pub struct FunctionFlattener<'a> {
  function_table: &'a FunctionTable,
}

impl<'a> FunctionFlattener<'a> {
  pub fn new(function_table: &'a FunctionTable) -> Self {
    Self { function_table }
  }
}

fn flatten_nested(function_name: &str, args: Vec<Expr>) -> Vec<Expr> {
  let mut new_args = Vec::with_capacity(args.len());
  for arg in args {
    match arg {
      Expr::Call(f, sub_args) => {
        if f == function_name {
          new_args.extend(flatten_nested(function_name, sub_args));
        } else {
          new_args.push(Expr::Call(f, sub_args));
        }
      }
      _ => {
        new_args.push(arg);
      }
    }
  }
  new_args
}

impl Simplifier for FunctionFlattener<'_> {
  fn simplify_expr_part(&self, expr: Expr, _ctx: &mut SimplifierContext) -> Expr {
    match expr {
      Expr::Call(function_name, args) => {
        let Some(known_function) = self.function_table.get(&function_name) else {
          return Expr::Call(function_name, args);
        };
        if known_function.flags().contains(FunctionFlags::PERMITS_FLATTENING) {
          let args = flatten_nested(&function_name, args);
          Expr::Call(function_name, args)
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
