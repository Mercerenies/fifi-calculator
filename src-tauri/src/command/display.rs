
//! Commands which manipulate the display settings.
//!
//! Most of these commands ignore the modifiers and numerical
//! argument, so the simpler commands are simply [`GeneralCommand`]
//! instances.

use super::base::{Command, CommandOutput};
use super::general::GeneralCommand;
use super::arguments::{ArgumentSchema, NullaryArgumentSchema};
use crate::undo::UndoableChange;
use crate::state::UndoableState;

/// [`UndoableChange`] which toggles the graphics enabled state.
#[derive(Clone, Debug)]
pub struct ToggleGraphicsChange;

pub fn toggle_graphics_command() -> impl Command + Send + Sync {
  GeneralCommand::new(|state, args, _| {
    NullaryArgumentSchema::new().validate(args)?;
    state.undo_stack_mut().push_cut();
    state.undo_stack_mut().push_change(ToggleGraphicsChange);
    let settings = state.display_settings_mut();
    settings.is_graphics_enabled = !settings.is_graphics_enabled;
    Ok(CommandOutput::success())
  })
}

impl UndoableChange<UndoableState> for ToggleGraphicsChange {
  fn play_forward(&self, state: &mut UndoableState) {
    let settings = state.display_settings_mut();
    settings.is_graphics_enabled = !settings.is_graphics_enabled;
  }

  fn play_backward(&self, state: &mut UndoableState) {
    let settings = state.display_settings_mut();
    settings.is_graphics_enabled = !settings.is_graphics_enabled;
  }

  fn undo_summary(&self) -> String {
    format!("{:?}", self)
  }
}
