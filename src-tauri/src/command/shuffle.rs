
//! Commands for shuffling the stack.

use super::base::{Command, CommandContext};
use crate::state::ApplicationState;
use crate::error::Error;
use crate::stack::shuffle;

/// Pops and discards a single value.
#[derive(Debug, Clone)]
pub struct PopCommand;

/// Swaps the top two stack values.
#[derive(Debug, Clone)]
pub struct SwapCommand;

/// Duplicates the top stack value.
#[derive(Debug, Clone)]
pub struct DupCommand;

impl Command for PopCommand {
  fn run_command(&self, state: &mut ApplicationState, ctx: &CommandContext) -> Result<(), Error> {
    // TODO Use context
    let _ = shuffle::pop_one(&mut state.main_stack)?;
    Ok(())
  }
}

impl Command for SwapCommand {
  fn run_command(&self, state: &mut ApplicationState, ctx: &CommandContext) -> Result<(), Error> {
    // TODO Use context
    let (a, b) = shuffle::pop_two(&mut state.main_stack)?;
    state.main_stack.push(b);
    state.main_stack.push(a);
    Ok(())
  }
}

impl Command for DupCommand {
  fn run_command(&self, state: &mut ApplicationState, ctx: &CommandContext) -> Result<(), Error> {
    // TODO Use context
    let a = shuffle::pop_one(&mut state.main_stack)?;
    state.main_stack.push(a.clone());
    state.main_stack.push(a);
    Ok(())
  }
}
