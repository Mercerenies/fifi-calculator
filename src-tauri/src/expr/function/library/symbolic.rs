
//! Functions for performing symbolic manipulation.

use crate::expr::function::Function;
use crate::expr::function::builder::{self, FunctionBuilder};
use crate::expr::function::table::FunctionTable;
use crate::expr::prisms;
use crate::util::prism::Identity;

pub fn append_symbolic_functions(table: &mut FunctionTable) {
  table.insert(substitute_function());
}

/// Replaces all instances of the needle variable with the given
/// replacement expression in the haystack.
///
/// It is NOT an error for the variable to be absent from the target
/// stack expression. In that case, the stack value is unchanged. This
/// command is also inherently single-pass, so a substitution can be
/// self-referencing. That is, it's meaningful to replace `x` with `x
/// + 1` using this function, since the `x` on the right-hand side
/// will not get recursively substituted.
pub fn substitute_function() -> Function {
  FunctionBuilder::new("substitute")
    .add_case(
      builder::arity_three().of_types(Identity::new(), prisms::ExprToVar, Identity::new())
        .and_then(|haystack, needle, replacement, _| {
          Ok(haystack.substitute_var(needle, replacement))
        })
    )
    .build()
}

