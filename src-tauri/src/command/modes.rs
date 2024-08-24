
//! Commands which manipulate the display settings.
//!
//! Most of these commands ignore the modifiers and numerical
//! argument, so the simpler commands are simply [`GeneralCommand`]
//! instances.

use super::base::{Command, CommandOutput, CommandContext};
use super::general::GeneralCommand;
use super::options::CommandOptions;
use super::subcommand::Subcommand;
use super::arguments::{NullaryArgumentSchema, UnaryArgumentSchema, validate_schema};
use crate::undo::UndoableChange;
use crate::state::{ApplicationState, UndoableState};
use crate::state::undo::ToggleFlagChange;
use crate::util::radix::{Radix, StringToRadix};
use crate::mode::display::language::LanguageMode;
use crate::mode::display::language::basic::BasicLanguageMode;
use crate::mode::display::language::fancy::FancyLanguageMode;

use std::sync::Arc;

/// [`UndoableChange`] which changes the display settings' preferred
/// radix to a given value.
#[derive(Clone, Debug)]
pub struct SetDisplayRadixChange {
  pub old_value: Radix,
  pub new_value: Radix,
}

/// [`UndoableChange`] which sets the engine's language mode to the
/// given value.
#[derive(Clone)]
pub struct SetLanguageModeChange {
  pub old_value: Arc<dyn LanguageMode + Send + Sync>,
  pub new_value: Arc<dyn LanguageMode + Send + Sync>,
}

/// Command which sets the display radix to the given value. Expects a
/// single radix value (per [`StringToRadix`]) as argument. Does not
/// use the keep modifier or numerical argument.
#[derive(Debug, Clone, Default)]
pub struct SetDisplayRadixCommand {
  _priv: (),
}

/// Command which sets the language mode to the given value. Does not
/// use the keep modifier or numerical argument.
#[derive(Clone)]
pub struct SetLanguageModeCommand {
  value: Arc<dyn LanguageMode + Send + Sync>,
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

impl SetLanguageModeCommand {
  pub fn new(value: Arc<dyn LanguageMode + Send + Sync>) -> Self {
    Self { value }
  }

  pub fn basic_language_mode() -> Self {
    let mode = Arc::new(BasicLanguageMode::from_common_operators());
    Self::new(mode)
  }

  pub fn fancy_language_mode() -> Self {
    let mode = Arc::new(FancyLanguageMode::from_common_unicode(
      BasicLanguageMode::from_common_operators().with_fancy_parens(),
    ));
    Self::new(mode)
  }
}

pub fn toggle_graphics_command() -> impl Command + Send + Sync {
  fn toggle_flag_change() -> ToggleFlagChange {
    ToggleFlagChange::from_accessor("is_graphics_enabled", |state| &mut state.display_settings_mut().is_graphics_enabled)
  }

  GeneralCommand::new(|state, args, _| {
    validate_schema(&NullaryArgumentSchema::new(), args)?;
    state.undo_stack_mut().push_cut();
    state.undo_stack_mut().push_change(toggle_flag_change());
    let settings = state.display_settings_mut();
    settings.is_graphics_enabled = !settings.is_graphics_enabled;
    Ok(CommandOutput::success())
  })
}

pub fn toggle_unicode_command() -> impl Command + Send + Sync {
  fn toggle_flag_change() -> ToggleFlagChange {
    ToggleFlagChange::from_accessor("prefers_unicode_output", |state| {
      &mut state.display_settings_mut().language_settings.prefers_unicode_output
    })
  }

  GeneralCommand::new(|state, args, _| {
    validate_schema(&NullaryArgumentSchema::new(), args)?;
    state.undo_stack_mut().push_cut();
    state.undo_stack_mut().push_change(toggle_flag_change());
    let settings = &mut state.display_settings_mut().language_settings;
    settings.prefers_unicode_output = !settings.prefers_unicode_output;
    Ok(CommandOutput::success())
  })
}

pub fn toggle_infinity_command() -> impl Command + Send + Sync {
  fn toggle_flag_change() -> ToggleFlagChange {
    ToggleFlagChange::from_getter_setter(
      "infinity_flag",
      |state| state.calculation_mode().has_infinity_flag(),
      |state, v| state.calculation_mode_mut().set_infinity_flag(v),
    )
  }

  GeneralCommand::new(|state, args, _| {
    validate_schema(&NullaryArgumentSchema::new(), args)?;
    state.undo_stack_mut().push_cut();
    state.undo_stack_mut().push_change(toggle_flag_change());
    let calc = state.calculation_mode_mut();
    calc.set_infinity_flag(!calc.has_infinity_flag());
    Ok(CommandOutput::success())
  })
}

pub fn toggle_fractional_command() -> impl Command + Send + Sync {
  fn toggle_flag_change() -> ToggleFlagChange {
    ToggleFlagChange::from_getter_setter(
      "fractional_flag",
      |state| state.calculation_mode().has_fractional_flag(),
      |state, v| state.calculation_mode_mut().set_fractional_flag(v),
    )
  }

  GeneralCommand::new(|state, args, _| {
    validate_schema(&NullaryArgumentSchema::new(), args)?;
    state.undo_stack_mut().push_cut();
    state.undo_stack_mut().push_change(toggle_flag_change());
    let calc = state.calculation_mode_mut();
    calc.set_fractional_flag(!calc.has_fractional_flag());
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
    let new_radix = validate_schema(&Self::argument_schema(), args)?;
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

impl Command for SetLanguageModeCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    _: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    validate_schema(&NullaryArgumentSchema::new(), args)?;
    let old_language_mode = state.display_settings().base_language_mode.clone();

    state.undo_stack_mut().push_cut();
    state.undo_stack_mut()
      .push_change(SetLanguageModeChange {
        old_value: old_language_mode,
        new_value: self.value.clone(),
      });
    state.display_settings_mut().base_language_mode = self.value.clone();
    Ok(CommandOutput::success())
  }

  fn as_subcommand(&self, _opts: &CommandOptions) -> Option<Subcommand> {
    None
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

impl UndoableChange<UndoableState> for SetLanguageModeChange {
  fn play_forward(&self, state: &mut UndoableState) {
    let settings = state.display_settings_mut();
    settings.base_language_mode = self.new_value.clone();
  }

  fn play_backward(&self, state: &mut UndoableState) {
    let settings = state.display_settings_mut();
    settings.base_language_mode = self.old_value.clone();
  }

  fn undo_summary(&self) -> String {
    String::from("SetLanguageModeChange { ... }")
  }
}
