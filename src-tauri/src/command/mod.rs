
pub mod algebra;
pub mod arguments;
mod base;
pub mod calculus;
pub mod dispatch;
pub mod display;
pub mod functional;
pub mod general;
pub mod graphics;
pub mod input;
pub mod nullary;
pub mod options;
pub mod shuffle;
pub mod units;
pub mod variables;
pub mod vector;

pub use base::{Command, CommandContext, CommandOutput};
use functional::{UnaryFunctionCommand, BinaryFunctionCommand};
use dispatch::CommandDispatchTable;
use crate::expr::Expr;
use crate::expr::number::ComplexNumber;
use crate::state::ApplicationState;

use std::collections::HashMap;

pub fn default_dispatch_table() -> CommandDispatchTable {
  let mut map: HashMap<String, Box<dyn Command + Send + Sync>> = HashMap::new();

  // TODO: We could probably get several of these automatically from
  // the function table. That would be nice.

  // Nullary commands
  map.insert("nop".to_string(), Box::new(nullary::NullaryCommand));
  map.insert("+".to_string(), Box::new(BinaryFunctionCommand::named("+")));
  map.insert("-".to_string(), Box::new(BinaryFunctionCommand::named("-")));
  map.insert("*".to_string(), Box::new(BinaryFunctionCommand::named("*")));
  map.insert("/".to_string(), Box::new(BinaryFunctionCommand::named("/")));
  map.insert("%".to_string(), Box::new(BinaryFunctionCommand::named("%")));
  map.insert("div".to_string(), Box::new(BinaryFunctionCommand::named("div")));
  map.insert("^".to_string(), Box::new(BinaryFunctionCommand::named("^")));
  map.insert("ln".to_string(), Box::new(UnaryFunctionCommand::named("ln")));
  map.insert("log".to_string(), Box::new(BinaryFunctionCommand::named("log")));
  map.insert("log10".to_string(), Box::new(UnaryFunctionCommand::new(log10))); // Currently unused
  map.insert("log2".to_string(), Box::new(UnaryFunctionCommand::new(log2))); // Currently unused
  map.insert("*i".to_string(), Box::new(UnaryFunctionCommand::new(times_i)));
  map.insert("e^".to_string(), Box::new(UnaryFunctionCommand::named("exp")));
  map.insert("negate".to_string(), Box::new(UnaryFunctionCommand::new(times_minus_one)));
  map.insert("pop".to_string(), Box::new(shuffle::PopCommand));
  map.insert("swap".to_string(), Box::new(shuffle::SwapCommand));
  map.insert("dup".to_string(), Box::new(shuffle::DupCommand));
  map.insert("substitute_vars".to_string(), Box::new(UnaryFunctionCommand::with_state(substitute_vars)));
  map.insert("pack".to_string(), Box::new(vector::PackCommand::new()));
  map.insert("unpack".to_string(), Box::new(vector::UnpackCommand::new()));
  map.insert("repeat".to_string(), Box::new(vector::RepeatCommand::new()));
  map.insert("vconcat".to_string(), Box::new(BinaryFunctionCommand::named("vconcat")));
  map.insert("iota".to_string(), Box::new(UnaryFunctionCommand::named("iota")));
  map.insert("head".to_string(), Box::new(UnaryFunctionCommand::named("head")));
  map.insert("cons".to_string(), Box::new(BinaryFunctionCommand::named("cons").assoc_right()));
  map.insert("abs".to_string(), Box::new(UnaryFunctionCommand::named("abs")));
  map.insert("signum".to_string(), Box::new(UnaryFunctionCommand::named("signum")));
  map.insert("conj".to_string(), Box::new(UnaryFunctionCommand::named("conj")));
  map.insert("arg".to_string(), Box::new(UnaryFunctionCommand::named("arg")));
  map.insert("re".to_string(), Box::new(UnaryFunctionCommand::named("re")));
  map.insert("im".to_string(), Box::new(UnaryFunctionCommand::named("im")));
  map.insert("lowercase".to_string(), Box::new(UnaryFunctionCommand::named("lowercase")));
  map.insert("uppercase".to_string(), Box::new(UnaryFunctionCommand::named("uppercase")));
  map.insert("remove_units".to_string(), Box::new(units::remove_units_command()));
  map.insert("extract_units".to_string(), Box::new(units::extract_units_command()));
  map.insert("=".to_string(), Box::new(BinaryFunctionCommand::named("=")));
  map.insert("!=".to_string(), Box::new(BinaryFunctionCommand::named("!=")));
  map.insert("<".to_string(), Box::new(BinaryFunctionCommand::named("<")));
  map.insert("<=".to_string(), Box::new(BinaryFunctionCommand::named("<=")));
  map.insert(">".to_string(), Box::new(BinaryFunctionCommand::named(">")));
  map.insert(">=".to_string(), Box::new(BinaryFunctionCommand::named(">=")));
  map.insert("..".to_string(), Box::new(BinaryFunctionCommand::named("..")));
  map.insert("..^".to_string(), Box::new(BinaryFunctionCommand::named("..^")));
  map.insert("^..".to_string(), Box::new(BinaryFunctionCommand::named("^..")));
  map.insert("^..^".to_string(), Box::new(BinaryFunctionCommand::named("^..^")));
  map.insert("plot".to_string(), Box::new(graphics::PlotCommand::new()));
  map.insert("contourplot".to_string(), Box::new(graphics::ContourPlotCommand::new()));
  map.insert("xy".to_string(), Box::new(BinaryFunctionCommand::named("xy")));
  map.insert("toggle_graphics".to_string(), Box::new(display::toggle_graphics_command()));

  // Commands which accept a single string.
  map.insert("push_number".to_string(), Box::new(input::push_number_command()));
  map.insert("push_expr".to_string(), Box::new(input::push_expr_command()));
  map.insert("push_string".to_string(), Box::new(input::push_string_command()));

  // Variable-related commands
  map.insert("manual_substitute".to_string(), Box::new(variables::SubstituteVarCommand::new()));
  map.insert("store_var".to_string(), Box::new(variables::StoreVarCommand::new()));
  map.insert("unbind_var".to_string(), Box::new(variables::UnbindVarCommand::new()));
  map.insert("deriv".to_string(), Box::new(calculus::DerivativeCommand::new()));
  map.insert("find_root".to_string(), Box::new(algebra::FindRootCommand::new()));

  // Specialized commands
  map.insert("mouse_move_stack_elem".to_string(), Box::new(shuffle::MoveStackElemCommand));
  map.insert("mouse_replace_stack_elem".to_string(), Box::new(shuffle::ReplaceStackElemCommand { is_mouse_interaction: true }));
  map.insert("replace_stack_elem".to_string(), Box::new(shuffle::ReplaceStackElemCommand { is_mouse_interaction: false }));

  CommandDispatchTable::from_hash_map(map)
}

fn log10(expr: Expr) -> Expr {
  Expr::call("log", vec![expr, Expr::from(10)])
}

fn log2(expr: Expr) -> Expr {
  Expr::call("log", vec![expr, Expr::from(2)])
}

fn times_i(expr: Expr) -> Expr {
  let ii = ComplexNumber::ii();
  Expr::call("*", vec![expr, Expr::from(ii)])
}

fn times_minus_one(expr: Expr) -> Expr {
  Expr::call("*", vec![expr, Expr::from(-1)])
}

fn substitute_vars(expr: Expr, state: &ApplicationState) -> Expr {
  let var_table = state.variable_table();
  expr.substitute_vars(var_table)
}

#[cfg(test)]
pub(crate) mod test_utils {
  use super::*;
  use crate::expr::Expr;
  use crate::command::options::CommandOptions;
  use crate::state::test_utils::state_for_stack;
  use crate::stack::test_utils::stack_of;
  use crate::stack::Stack;

  /// Trait for arguments which are acceptable to pass to
  /// `act_on_stack`. This trait merely exists to allow overloading
  /// that test helper, so that we don't have to explicitly construct
  /// a default command context all over the place.
  pub trait ActOnStackArg {
    fn mutate_arg(self, args: &mut Vec<String>, context: &mut CommandContext);
  }

  impl ActOnStackArg for () {
    fn mutate_arg(self, _args: &mut Vec<String>, _context: &mut CommandContext) {}
  }

  impl<A: ActOnStackArg, B: ActOnStackArg> ActOnStackArg for (A, B) {
    fn mutate_arg(self, args: &mut Vec<String>, context: &mut CommandContext) {
      let (a, b) = self;
      a.mutate_arg(args, context);
      b.mutate_arg(args, context);
    }
  }

  impl ActOnStackArg for CommandOptions {
    fn mutate_arg(self, _args: &mut Vec<String>, context: &mut CommandContext) {
      context.opts = self;
    }
  }

  impl<S> ActOnStackArg for Vec<S>
  where S: Into<String> {
    fn mutate_arg(self, args: &mut Vec<String>, _context: &mut CommandContext) {
      *args = self.into_iter().map(|s| s.into()).collect();
    }
  }

  impl<F> ActOnStackArg for F
  where F: FnOnce(&mut Vec<String>, &mut CommandContext) {
    fn mutate_arg(self, args: &mut Vec<String>, context: &mut CommandContext) {
      self(args, context)
    }
  }

  /// Tests the operation on the given input stack. Passes no string
  /// arguments. If the result is an error, this function additionally
  /// asserts that the stack is unchanged.
  pub fn act_on_stack<E, A>(
    command: &impl Command,
    command_modifier: A,
    input_stack: Vec<E>,
  ) -> Result<Stack<Expr>, anyhow::Error>
  where E: Into<Expr> + Clone,
        A: ActOnStackArg {
    let mut state = state_for_stack(input_stack.clone());
    let mut args = Vec::new();
    let mut context = CommandContext::default();
    command_modifier.mutate_arg(&mut args, &mut context);
    match command.run_command(&mut state, args, &context) {
      Ok(_) => {
        Ok(state.into_main_stack())
      }
      Err(err) => {
        assert_eq!(state.into_main_stack(), stack_of(input_stack));
        Err(err)
      }
    }
  }
}
