
pub mod accum;
pub mod algebra;
pub mod arguments;
mod base;
pub mod calculus;
pub mod datetime;
pub mod dispatch;
pub mod flag_dispatch;
pub mod functional;
pub mod general;
pub mod graphics;
pub mod input;
pub mod modes;
pub mod nullary;
pub mod options;
pub mod shuffle;
pub mod statistics;
pub mod subcommand;
pub mod units;
pub mod variables;
pub mod vector;

pub use base::{Command, CommandContext, CommandOutput};
use functional::{PushConstantCommand, UnaryFunctionCommand, BinaryFunctionCommand};
use statistics::DatasetDrivenCommand;
use dispatch::CommandDispatchTable;
use flag_dispatch::{FlagDispatchArgs, dispatch_on_flags_command,
                    dispatch_on_inverse_command, dispatch_on_hyper_command};
use crate::expr::Expr;
use crate::expr::simplifier::numerical::simplify_numerically;
use crate::expr::walker::postorder_walk_ok;
use crate::expr::number::ComplexNumber;
use crate::expr::algebra::infinity::InfiniteConstant;
use crate::expr::incomplete::{IncompleteObject, ObjectType};
use crate::state::ApplicationState;

use std::collections::HashMap;

pub fn default_dispatch_table() -> CommandDispatchTable {
  let mut map: HashMap<String, Box<dyn Command + Send + Sync>> = HashMap::new();

  // TODO: We could probably get several of these automatically from
  // the function table. That would be nice.

  // Basc arithmetic (no arguments)
  map.insert("nop".to_string(), Box::new(nullary::NullaryCommand));
  map.insert("+".to_string(), Box::new(BinaryFunctionCommand::named("+")));
  map.insert("-".to_string(), Box::new(BinaryFunctionCommand::named("-")));
  map.insert("*".to_string(), Box::new(BinaryFunctionCommand::named("*")));
  map.insert("/".to_string(), Box::new(BinaryFunctionCommand::named("/")));
  map.insert("%".to_string(), Box::new(BinaryFunctionCommand::named("%")));
  map.insert("div".to_string(), Box::new(BinaryFunctionCommand::named("div")));
  map.insert("^".to_string(), Box::new(dispatch_on_inverse_command(
    BinaryFunctionCommand::named("^"),
    BinaryFunctionCommand::new(nroot),
  )));
  map.insert("ln".to_string(), Box::new(dispatch_on_flags_command(FlagDispatchArgs {
    no_flags: UnaryFunctionCommand::named("ln"),
    hyper_flag: UnaryFunctionCommand::new(log10),
    inv_flag: UnaryFunctionCommand::named("exp"),
    inv_hyper_flag: UnaryFunctionCommand::new(pow10),
  })));
  map.insert("log".to_string(), Box::new(dispatch_on_inverse_command(
    BinaryFunctionCommand::named("log"),
    BinaryFunctionCommand::new(pow_flipped),
  )));
  map.insert("*i".to_string(), Box::new(dispatch_on_inverse_command(
    UnaryFunctionCommand::new(times_i),
    UnaryFunctionCommand::new(div_i),
  )));
  map.insert("e^".to_string(), Box::new(dispatch_on_flags_command(FlagDispatchArgs {
    no_flags: UnaryFunctionCommand::named("exp"),
    hyper_flag: UnaryFunctionCommand::new(pow10),
    inv_flag: UnaryFunctionCommand::named("ln"),
    inv_hyper_flag: UnaryFunctionCommand::new(log10),
  })));
  map.insert("sqrt".to_string(), Box::new(dispatch_on_flags_command(FlagDispatchArgs {
    no_flags: UnaryFunctionCommand::named("sqrt"),
    hyper_flag: UnaryFunctionCommand::new(log2),
    inv_flag: UnaryFunctionCommand::new(pow2),
    inv_hyper_flag: UnaryFunctionCommand::new(pow2),
  })));
  map.insert("negate".to_string(), Box::new(UnaryFunctionCommand::new(times_minus_one)));
  map.insert("recip".to_string(), Box::new(UnaryFunctionCommand::named("recip")));
  map.insert("min".to_string(), Box::new(BinaryFunctionCommand::named("min")));
  map.insert("max".to_string(), Box::new(BinaryFunctionCommand::named("max")));

  // Trigonometry
  map.insert("sin".to_string(), Box::new(dispatch_on_flags_command(FlagDispatchArgs {
    no_flags: UnaryFunctionCommand::named("sin"),
    hyper_flag: UnaryFunctionCommand::named("sinh"),
    inv_flag: UnaryFunctionCommand::named("asin"),
    inv_hyper_flag: UnaryFunctionCommand::named("asinh"),
  })));
  map.insert("cos".to_string(), Box::new(dispatch_on_flags_command(FlagDispatchArgs {
    no_flags: UnaryFunctionCommand::named("cos"),
    hyper_flag: UnaryFunctionCommand::named("cosh"),
    inv_flag: UnaryFunctionCommand::named("acos"),
    inv_hyper_flag: UnaryFunctionCommand::named("acosh"),
  })));
  map.insert("tan".to_string(), Box::new(dispatch_on_flags_command(FlagDispatchArgs {
    no_flags: UnaryFunctionCommand::named("tan"),
    hyper_flag: UnaryFunctionCommand::named("tanh"),
    inv_flag: UnaryFunctionCommand::named("atan"),
    inv_hyper_flag: UnaryFunctionCommand::named("atanh"),
  })));

  // Stack shuffling (no arguments)
  map.insert("pop".to_string(), Box::new(shuffle::PopCommand));
  map.insert("swap".to_string(), Box::new(shuffle::SwapCommand));
  map.insert("dup".to_string(), Box::new(shuffle::DupCommand));

  // Constructors (no arguments)
  map.insert("..".to_string(), Box::new(BinaryFunctionCommand::named("..")));
  map.insert("..^".to_string(), Box::new(BinaryFunctionCommand::named("..^")));
  map.insert("^..".to_string(), Box::new(BinaryFunctionCommand::named("^..")));
  map.insert("^..^".to_string(), Box::new(BinaryFunctionCommand::named("^..^")));

  // Incomplete object handling
  map.insert("incomplete[".to_string(), Box::new(PushConstantCommand::new(IncompleteObject::new(ObjectType::LeftBracket))));
  map.insert("incomplete(".to_string(), Box::new(PushConstantCommand::new(IncompleteObject::new(ObjectType::LeftParen))));
  map.insert("incomplete]".to_string(), Box::new(vector::VectorFromIncompleteObjectCommand::new()));
  map.insert("incomplete)".to_string(), Box::new(vector::ComplexFromIncompleteObjectCommand::new()));

  // Constants (no arguments)
  map.insert("infinity".to_string(), Box::new(PushConstantCommand::new(InfiniteConstant::PosInfinity)));
  map.insert("neg_infinity".to_string(), Box::new(PushConstantCommand::new(InfiniteConstant::NegInfinity)));
  map.insert("undir_infinity".to_string(), Box::new(PushConstantCommand::new(InfiniteConstant::UndirInfinity)));
  map.insert("nan_infinity".to_string(), Box::new(PushConstantCommand::new(InfiniteConstant::NotANumber)));
  map.insert("pi".to_string(), Box::new(PushConstantCommand::new(Expr::var("pi").unwrap())));
  map.insert("gamma".to_string(), Box::new(PushConstantCommand::new(Expr::var("gamma").unwrap())));
  map.insert("e".to_string(), Box::new(PushConstantCommand::new(Expr::var("e").unwrap())));
  map.insert("phi".to_string(), Box::new(PushConstantCommand::new(Expr::var("phi").unwrap())));

  // Other nullary
  map.insert("substitute_vars".to_string(), Box::new(UnaryFunctionCommand::with_state(substitute_vars)));
  map.insert("substitute_numerically".to_string(), Box::new(UnaryFunctionCommand::with_state(substitute_and_numerical_simplify)));
  map.insert("pack".to_string(), Box::new(vector::PackCommand::new()));
  map.insert("unpack".to_string(), Box::new(vector::UnpackCommand::new()));
  map.insert("repeat".to_string(), Box::new(vector::RepeatCommand::new()));
  map.insert("diag".to_string(), Box::new(vector::DiagonalCommand::new()));
  map.insert("vconcat".to_string(), Box::new(BinaryFunctionCommand::named("vconcat")));
  map.insert("iota".to_string(), Box::new(UnaryFunctionCommand::named("iota")));
  map.insert("head".to_string(), Box::new(dispatch_on_flags_command(FlagDispatchArgs {
    no_flags: UnaryFunctionCommand::named("head"),
    hyper_flag: UnaryFunctionCommand::named("last"),
    inv_flag: UnaryFunctionCommand::named("tail"),
    inv_hyper_flag: UnaryFunctionCommand::named("init"),
  })));
  map.insert("cons".to_string(), Box::new(dispatch_on_hyper_command(
    BinaryFunctionCommand::named("cons").assoc_right(),
    BinaryFunctionCommand::named("snoc"),
  )));
  map.insert("abs".to_string(), Box::new(UnaryFunctionCommand::named("abs")));
  map.insert("signum".to_string(), Box::new(UnaryFunctionCommand::named("signum")));
  map.insert("conj".to_string(), Box::new(UnaryFunctionCommand::named("conj")));
  map.insert("arg".to_string(), Box::new(UnaryFunctionCommand::named("arg")));
  map.insert("re".to_string(), Box::new(UnaryFunctionCommand::named("re")));
  map.insert("im".to_string(), Box::new(UnaryFunctionCommand::named("im")));
  map.insert("lowercase".to_string(), Box::new(UnaryFunctionCommand::named("lowercase")));
  map.insert("uppercase".to_string(), Box::new(UnaryFunctionCommand::named("uppercase")));
  map.insert("=".to_string(), Box::new(BinaryFunctionCommand::named("=")));
  map.insert("!=".to_string(), Box::new(BinaryFunctionCommand::named("!=")));
  map.insert("<".to_string(), Box::new(BinaryFunctionCommand::named("<")));
  map.insert("<=".to_string(), Box::new(BinaryFunctionCommand::named("<=")));
  map.insert(">".to_string(), Box::new(BinaryFunctionCommand::named(">")));
  map.insert(">=".to_string(), Box::new(BinaryFunctionCommand::named(">=")));
  map.insert("plot".to_string(), Box::new(graphics::PlotCommand::new()));
  map.insert("contourplot".to_string(), Box::new(graphics::ContourPlotCommand::new()));
  map.insert("xy".to_string(), Box::new(BinaryFunctionCommand::named("xy")));

  // Mode commands
  map.insert("toggle_graphics".to_string(), Box::new(modes::toggle_graphics_command()));
  map.insert("toggle_unicode".to_string(), Box::new(modes::toggle_unicode_command()));
  map.insert("toggle_infinity".to_string(), Box::new(modes::toggle_infinity_command()));
  map.insert("toggle_fractional".to_string(), Box::new(modes::toggle_fractional_command()));
  map.insert("set_basic_language_mode".to_string(), Box::new(modes::SetLanguageModeCommand::basic_language_mode()));
  map.insert("set_fancy_language_mode".to_string(), Box::new(modes::SetLanguageModeCommand::fancy_language_mode()));

  // Unit conversion
  map.insert("simplify_units".to_string(), Box::new(units::simplify_units_command()));
  map.insert("remove_units".to_string(), Box::new(units::remove_units_command()));
  map.insert("extract_units".to_string(), Box::new(units::extract_units_command()));
  map.insert("convert_units".to_string(), Box::new(units::ConvertUnitsCommand::new()));
  map.insert("convert_units_with_context".to_string(), Box::new(units::ContextualConvertUnitsCommand::new()));
  map.insert("convert_temp".to_string(), Box::new(units::ConvertTemperatureCommand::new()));
  map.insert("convert_temp_with_context".to_string(), Box::new(units::ContextualConvertTemperatureCommand::new()));

  // Datetime commands
  map.insert("days_since_zero".to_string(), Box::new(datetime::days_since_command(datetime::ZERO_DATE)));
  map.insert("julian_day".to_string(), Box::new(datetime::days_since_command(datetime::ZERO_JULIAN_DAY)));
  map.insert("unix_time".to_string(), Box::new(datetime::secs_since_command(datetime::UNIX_EPOCH)));
  map.insert("now".to_string(), Box::new(datetime::NowCommand));
  map.insert("convert_tz".to_string(), Box::new(datetime::ConvertTimezoneCommand));

  // Vector commands
  map.insert("subvector".to_string(), Box::new(dispatch_on_hyper_command(
    vector::subvector_command(),
    vector::remove_subvector_command(),
  )));
  map.insert("length".to_string(), Box::new(dispatch_on_hyper_command(
    UnaryFunctionCommand::named("length"),
    UnaryFunctionCommand::named("shape"),
  )));
  map.insert("find".to_string(), Box::new(BinaryFunctionCommand::named("find")));
  map.insert("arrange".to_string(), Box::new(vector::arrange_vector_command()));
  map.insert("sort".to_string(), Box::new(dispatch_on_inverse_command(
    UnaryFunctionCommand::named("sort"),
    UnaryFunctionCommand::named("rsort"),
  )));
  map.insert("grade".to_string(), Box::new(dispatch_on_inverse_command(
    UnaryFunctionCommand::named("grade"),
    UnaryFunctionCommand::named("rgrade"),
  )));
  map.insert("reverse".to_string(), Box::new(UnaryFunctionCommand::named("reverse")));
  map.insert("vmask".to_string(), Box::new(BinaryFunctionCommand::named("vmask")));
  map.insert("norm".to_string(), Box::new(vector::NormCommand::new()));
  map.insert("cross".to_string(), Box::new(BinaryFunctionCommand::named("cross")));
  map.insert("vapply".to_string(), Box::new(accum::VectorApplyCommand::new()));
  map.insert("vmap".to_string(), Box::new(accum::VectorMapCommand::new()));
  map.insert("vreduce".to_string(), Box::new(dispatch_on_inverse_command(
    accum::VectorReduceCommand::new(accum::ReduceDir::LeftToRight),
    accum::VectorReduceCommand::new(accum::ReduceDir::RightToLeft),
  )));
  map.insert("vaccum".to_string(), Box::new(dispatch_on_inverse_command(
    accum::VectorAccumCommand::new(accum::ReduceDir::LeftToRight),
    accum::VectorAccumCommand::new(accum::ReduceDir::RightToLeft),
  )));
  map.insert("outerprod".to_string(), Box::new(accum::OuterProductCommand::new()));
  map.insert("innerprod".to_string(), Box::new(accum::InnerProductCommand::new()));

  // Vector statistics commands
  map.insert("mean".to_string(), Box::new(dispatch_on_flags_command(FlagDispatchArgs {
    no_flags: DatasetDrivenCommand::named("mean"),
    hyper_flag: DatasetDrivenCommand::named("median"),
    inv_flag: DatasetDrivenCommand::named("gmean"),
    inv_hyper_flag: DatasetDrivenCommand::named("agmean"),
  })));
  map.insert("hmean".to_string(), Box::new(DatasetDrivenCommand::named("hmean")));
  map.insert("rms".to_string(), Box::new(DatasetDrivenCommand::named("rms")));
  map.insert("stddev".to_string(), Box::new(dispatch_on_flags_command(FlagDispatchArgs {
    no_flags: DatasetDrivenCommand::named("stddev"),
    hyper_flag: DatasetDrivenCommand::named("variance"),
    inv_flag: DatasetDrivenCommand::named("pstddev"),
    inv_hyper_flag: DatasetDrivenCommand::named("pvariance"),
  })));
  map.insert("covariance".to_string(), Box::new(dispatch_on_flags_command(FlagDispatchArgs {
    no_flags: statistics::sample_covar_command(),
    hyper_flag: statistics::correlation_command(),
    inv_flag: statistics::pop_covar_command(),
    inv_hyper_flag: statistics::correlation_command(), // Note: We may make this one do something different later
  })));

  // Matrix commands
  map.insert("identity_matrix".to_string(), Box::new(vector::IdentityMatrixCommand::new()));
  map.insert("nth".to_string(), Box::new(dispatch_on_hyper_command(
    vector::nth_element_command(),
    vector::remove_nth_element_command(),
  )));
  map.insert("nth_column".to_string(), Box::new(dispatch_on_hyper_command(
    vector::nth_column_command(),
    vector::remove_nth_column_command(),
  )));
  map.insert("transpose".to_string(), Box::new(dispatch_on_hyper_command(
    UnaryFunctionCommand::named("transpose"),
    UnaryFunctionCommand::new(conj_transpose),
  )));
  map.insert("det".to_string(), Box::new(UnaryFunctionCommand::named("det")));
  map.insert("trace".to_string(), Box::new(UnaryFunctionCommand::named("trace")));
  map.insert("@".to_string(), Box::new(BinaryFunctionCommand::named("@")));
  map.insert("kron".to_string(), Box::new(BinaryFunctionCommand::named("kron")));

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
  map.insert("set_display_radix".to_string(), Box::new(modes::SetDisplayRadixCommand::new()));

  CommandDispatchTable::from_hash_map(map)
}

fn log10(expr: Expr) -> Expr {
  Expr::call("log", vec![expr, Expr::from(10)])
}

fn log2(expr: Expr) -> Expr {
  Expr::call("log", vec![expr, Expr::from(2)])
}

fn pow10(expr: Expr) -> Expr {
  Expr::call("^", vec![Expr::from(10), expr])
}

fn pow2(expr: Expr) -> Expr {
  Expr::call("^", vec![Expr::from(2), expr])
}

fn pow_flipped(a: Expr, b: Expr) -> Expr {
  // Note: Flipped argument order
  Expr::call("^", vec![b, a])
}

fn nroot(a: Expr, b: Expr) -> Expr {
  Expr::call("^", vec![
    a,
    Expr::call("/", vec![Expr::from(1), b]),
  ])
}

fn times_i(expr: Expr) -> Expr {
  let ii = ComplexNumber::ii();
  Expr::call("*", vec![expr, Expr::from(ii)])
}

fn div_i(expr: Expr) -> Expr {
  let ii = ComplexNumber::ii();
  Expr::call("/", vec![expr, Expr::from(ii)])
}

fn times_minus_one(expr: Expr) -> Expr {
  Expr::call("*", vec![expr, Expr::from(-1)])
}

fn conj_transpose(expr: Expr) -> Expr {
  Expr::call("conj", vec![Expr::call("transpose", vec![expr])])
}

fn substitute_vars(expr: Expr, state: &ApplicationState) -> Expr {
  let var_table = state.variable_table();
  expr.substitute_vars(var_table)
}

fn numerical_simplify(expr: Expr) -> Expr {
  postorder_walk_ok(expr, simplify_numerically)
}

fn substitute_and_numerical_simplify(expr: Expr, state: &ApplicationState) -> Expr {
  let expr = substitute_vars(numerical_simplify(expr), state);
  numerical_simplify(expr)
}

#[cfg(test)]
pub(crate) mod test_utils {
  use super::*;
  use crate::expr::Expr;
  use crate::expr::simplifier::default_simplifier;
  use crate::expr::function::table::FunctionTable;
  use crate::expr::function::library::build_function_table;
  use crate::command::options::CommandOptions;
  use crate::state::test_utils::state_for_stack;
  use crate::stack::test_utils::stack_of;
  use crate::stack::Stack;
  use crate::mode::calculation::CalculationMode;

  use once_cell::sync::Lazy;

  /// Trait for arguments which are acceptable to pass to
  /// `act_on_stack`. This trait merely exists to allow overloading
  /// that test helper, so that we don't have to explicitly construct
  /// a default command context all over the place.
  pub trait ActOnStackArg {
    fn mutate_arg(self, args: &mut Vec<String>, state: &mut ApplicationState, context: &mut CommandContext);
  }

  impl ActOnStackArg for () {
    fn mutate_arg(self, _args: &mut Vec<String>, _state: &mut ApplicationState, _context: &mut CommandContext) {}
  }

  impl<A: ActOnStackArg, B: ActOnStackArg> ActOnStackArg for (A, B) {
    fn mutate_arg(self, args: &mut Vec<String>, state: &mut ApplicationState, context: &mut CommandContext) {
      let (a, b) = self;
      a.mutate_arg(args, state, context);
      b.mutate_arg(args, state, context);
    }
  }

  impl<A: ActOnStackArg, B: ActOnStackArg, C: ActOnStackArg> ActOnStackArg for (A, B, C) {
    fn mutate_arg(self, args: &mut Vec<String>, state: &mut ApplicationState, context: &mut CommandContext) {
      let (a, b, c) = self;
      a.mutate_arg(args, state, context);
      b.mutate_arg(args, state, context);
      c.mutate_arg(args, state, context);
    }
  }

  impl ActOnStackArg for CommandOptions {
    fn mutate_arg(self, _args: &mut Vec<String>, _state: &mut ApplicationState, context: &mut CommandContext) {
      context.opts = self;
    }
  }

  impl<S> ActOnStackArg for Vec<S>
  where S: Into<String> {
    fn mutate_arg(self, args: &mut Vec<String>, _state: &mut ApplicationState, _context: &mut CommandContext) {
      *args = self.into_iter().map(|s| s.into()).collect();
    }
  }

  impl ActOnStackArg for CalculationMode {
    fn mutate_arg(self, _args: &mut Vec<String>, state: &mut ApplicationState, _context: &mut CommandContext) {
      *state.calculation_mode_mut() = self;
    }
  }

  impl<F> ActOnStackArg for F
  where F: FnOnce(&mut Vec<String>, &mut ApplicationState, &mut CommandContext) {
    fn mutate_arg(self, args: &mut Vec<String>, state: &mut ApplicationState, context: &mut CommandContext) {
      self(args, state, context)
    }
  }

  /// Tests the command on the given input stack. If the command is
  /// successful, returns the new stack. If the command results in an
  /// error, this function asserts that the stack is unchanged and
  /// then returns the error.
  ///
  /// The `command_modifier` argument is overloaded by the trait
  /// [`ActOnStackArg`] and is designed to make writing tests that
  /// utilize this function easier. Broadly, `command_modifier` can be
  /// thought of as a `FnOnce(&mut Vec<String>, &mut CommandContext)`.
  /// That is, it's an arbitrary function which can mutate the
  /// argument list and the command context. `act_on_stack` first
  /// generates an empty argument list and a [`Default`] command
  /// context, then allows the modifier to modify it freely. In this
  /// way, commands which don't require any modification can merely
  /// pass `()`, while those that require some modification can pass
  /// only the parts that need modifying.
  ///
  /// Acceptable `command_modifier` types:
  ///
  /// * `()` - Performs no modifications.
  ///
  /// * [`CommandOptions`] - Replaces the options in the
  /// `CommandContext` with this value.
  ///
  /// * `Vec<impl Into<String>>` - Replaces the argument list with
  /// this value.
  ///
  /// * `FnOnce(&mut Vec<String>, &mut CommandContext)` -
  /// General-purpose case. Calls the function.
  ///
  /// Additionally, 2-tuples and 3-tuples of modifiers can be passed.
  /// In that case, the modifiers are run in order, one after the
  /// other.
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
    command_modifier.mutate_arg(&mut args, &mut state, &mut context);
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

  /// This function is an [`ActOnStackArg`] which sets up the basic
  /// simplifier. This is the simplifier used by default in the GUI.
  pub fn setup_default_simplifier(_args: &mut Vec<String>, _state: &mut ApplicationState, context: &mut CommandContext) {
    static FUNCTION_TABLE: Lazy<FunctionTable> = Lazy::new(build_function_table);
    context.simplifier = default_simplifier(&FUNCTION_TABLE);
  }
}
