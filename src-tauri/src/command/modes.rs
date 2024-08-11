
//! Commands which manipulate the display settings.
//!
//! Most of these commands ignore the modifiers and numerical
//! argument, so the simpler commands are simply [`GeneralCommand`]
//! instances.

use super::base::{Command, CommandOutput, CommandContext};
use super::general::GeneralCommand;
use super::options::CommandOptions;
use super::subcommand::Subcommand;
use super::arguments::{ArgumentSchema, NullaryArgumentSchema, UnaryArgumentSchema};
use crate::undo::UndoableChange;
use crate::state::{ApplicationState, UndoableState};
use crate::state::undo::ToggleFlagChange;
use crate::util::radix::{Radix, StringToRadix};

/// [`UndoableChange`] which toggles the infinity mode.
///
/// Note that this *cannot* be written in terms of
/// [`ToggleFlagChange`], as `ToggleInfinityChange` references an
/// individual bit on a bitmask, and `ToggleFlagChange` requires a
/// `&mut bool`.
#[derive(Clone, Debug)]
pub struct ToggleInfinityChange;

/// [`UndoableChange`] which changes the display settings' preferred
/// radix to a given value.
#[derive(Clone, Debug)]
pub struct SetDisplayRadixChange {
  pub old_value: Radix,
  pub new_value: Radix,
}

/// Command which sets the display radix to the given value. Expects a
/// single radix value (per [`StringToRadix`]) as argument. Does not
/// use the keep modifier or numerical argument.
#[derive(Debug, Clone, Default)]
pub struct SetDisplayRadixCommand {
  _priv: (),
}

impl SetDisplayRadixCommand {
  pub fn new() -> Self {
    Self { _priv: () }
  }

  fn argument_schema() -> UnaryArgumentSchema<StringToRadix, Radix> {
    UnaryArgumentSchema::new(
      String::from("valid numerical radix"),
      StringToRadix,
    )
  }
}

pub fn toggle_graphics_command() -> impl Command + Send + Sync {
  fn toggle_flag_change() -> ToggleFlagChange<fn(&mut UndoableState) -> &mut bool> {
    ToggleFlagChange::new("is_graphics_enabled", |state| &mut state.display_settings_mut().is_graphics_enabled)
  }

  GeneralCommand::new(|state, args, _| {
    NullaryArgumentSchema::new().validate(args)?;
    state.undo_stack_mut().push_cut();
    state.undo_stack_mut().push_change(toggle_flag_change());
    let settings = state.display_settings_mut();
    settings.is_graphics_enabled = !settings.is_graphics_enabled;
    Ok(CommandOutput::success())
  })
}

pub fn toggle_unicode_command() -> impl Command + Send + Sync {
  fn toggle_flag_change() -> ToggleFlagChange<fn(&mut UndoableState) -> &mut bool> {
    ToggleFlagChange::new("prefers_unicode_output", |state| {
      &mut state.display_settings_mut().language_settings.prefers_unicode_output
    })
  }

  GeneralCommand::new(|state, args, _| {
    NullaryArgumentSchema::new().validate(args)?;
    state.undo_stack_mut().push_cut();
    state.undo_stack_mut().push_change(toggle_flag_change());
    let settings = &mut state.display_settings_mut().language_settings;
    settings.prefers_unicode_output = !settings.prefers_unicode_output;
    Ok(CommandOutput::success())
  })
}

pub fn toggle_infinity_command() -> impl Command + Send + Sync {
  GeneralCommand::new(|state, args, _| {
    NullaryArgumentSchema::new().validate(args)?;
    state.undo_stack_mut().push_cut();
    state.undo_stack_mut().push_change(ToggleInfinityChange);
    let calc = state.calculation_mode_mut();
    calc.set_infinity_flag(!calc.has_infinity_flag());
    Ok(CommandOutput::success())
  })
}

impl Command for SetDisplayRadixCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    _: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    let old_radix = state.display_settings().language_settings.preferred_radix;
    let new_radix = Self::argument_schema().validate(args)?;
    if old_radix == new_radix {
      // Nothing to change, so don't modify the undo stack.
      return Ok(CommandOutput::success());
    }

    state.display_settings_mut().language_settings.preferred_radix = new_radix;
    state.undo_stack_mut().push_cut();
    state.undo_stack_mut()
      .push_change(SetDisplayRadixChange { old_value: old_radix, new_value: new_radix });
    Ok(CommandOutput::success())
  }

  fn as_subcommand(&self, _opts: &CommandOptions) -> Option<Subcommand> {
    None
  }
}

impl UndoableChange<UndoableState> for ToggleInfinityChange {
  fn play_forward(&self, state: &mut UndoableState) {
    let calc = state.calculation_mode_mut();
    calc.set_infinity_flag(!calc.has_infinity_flag());
  }

  fn play_backward(&self, state: &mut UndoableState) {
    let calc = state.calculation_mode_mut();
    calc.set_infinity_flag(!calc.has_infinity_flag());
  }

  fn undo_summary(&self) -> String {
    format!("{:?}", self)
  }
}

impl UndoableChange<UndoableState> for SetDisplayRadixChange {
  fn play_forward(&self, state: &mut UndoableState) {
    let settings = state.display_settings_mut();
    settings.language_settings.preferred_radix = self.new_value;
  }

  fn play_backward(&self, state: &mut UndoableState) {
    let settings = state.display_settings_mut();
    settings.language_settings.preferred_radix = self.old_value;
  }

  fn undo_summary(&self) -> String {
    format!("{:?}", self)
  }
}
