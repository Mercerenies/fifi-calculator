
//! Commands which manipulate the display settings.
//!
//! Most of these commands ignore the modifiers and numerical
//! argument, so the simpler commands are simply [`GeneralCommand`]
//! instances.

use super::base::{Command, CommandOutput};
use super::general::GeneralCommand;
use super::arguments::{ArgumentSchema, NullaryArgumentSchema};

pub fn toggle_graphics_command() -> impl Command + Send + Sync {
  GeneralCommand::new(|state, args, _| {
    NullaryArgumentSchema::new().validate(args)?;
    let settings = state.display_settings_mut();
    settings.is_graphics_enabled = !settings.is_graphics_enabled;
    Ok(CommandOutput::success())
  })
}
