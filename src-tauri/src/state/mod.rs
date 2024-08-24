
//! Backend application state manager.

pub mod delegate;
pub mod events;
pub mod modeline;
pub mod query;
pub mod tauri_command;
pub mod undo;
pub mod validation;

use events::{RefreshStackPayload, UndoAvailabilityPayload, ModelinePayload};
use delegate::UndoingDelegate;
use modeline::{ModelineBuilder, LanguageModeValue, boolean_flag};
use crate::stack::{Stack, DelegatingStack};
use crate::expr::Expr;
use crate::expr::function::table::FunctionTable;
use crate::expr::function::library::build_function_table;
use crate::expr::var::table::VarTable;
use crate::expr::var::constants::bind_constants;
use crate::expr::number::Number;
use crate::expr::algebra::term::TermParser;
use crate::command::default_dispatch_table;
use crate::command::dispatch::CommandDispatchTable;
use crate::mode::display::DisplaySettings;
use crate::mode::calculation::CalculationMode;
use crate::undo::{UndoStack, UndoError};
use crate::units::parsing::{UnitParser, default_parser};

use serde::{Serialize, Deserialize};

use tauri::Manager;

use std::sync::Mutex;

pub struct TauriApplicationState {
  pub state: Mutex<ApplicationState>,
  pub command_table: CommandDispatchTable,
  pub function_table: FunctionTable,
  pub units_parser: Box<dyn UnitParser<Number> + Send + Sync>,
}

#[derive(Default)]
pub struct ApplicationState {
  undoable_state: UndoableState,
  undo_stack: UndoStack<UndoableState>,
}

#[derive(Default)]
pub struct UndoableState {
  main_stack: Stack<Expr>,
  display_settings: DisplaySettings,
  calculation_mode: CalculationMode,
  variables: VarTable<Expr>,
}

/// Direction of an undo command issued to Tauri.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UndoDirection {
  Undo, Redo,
}

impl TauriApplicationState {
  pub fn with_default_tables() -> Self {
    let state = {
      let mut state = ApplicationState::default();
      bind_constants(state.variable_table_mut());
      state
    };
    Self {
      state: Mutex::new(state),
      command_table: default_dispatch_table(),
      function_table: build_function_table(),
      units_parser: Box::new(default_parser()),
    }
  }
}

impl ApplicationState {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn term_parser(&self) -> TermParser {
    // Note: Later, we will take the scalar mode into account when
    // constructing this value.
    TermParser::new()
  }

  pub fn send_refresh_stack_event(
    &self,
    app_handle: &tauri::AppHandle,
    force_scroll_down: bool,
  ) -> tauri::Result<()> {
    let state = &self.undoable_state;
    let displayed_stack: Vec<String> =
      state.main_stack.iter().map(|expr| state.display_settings.to_html(expr)).collect();
    let payload = RefreshStackPayload { stack: displayed_stack, force_scroll_down };
    app_handle.emit(RefreshStackPayload::EVENT_NAME, payload)
  }

  pub fn send_undo_buttons_event(&self, app_handle: &tauri::AppHandle) -> tauri::Result<()> {
    let payload = UndoAvailabilityPayload {
      has_undos: self.undo_stack.has_undos(),
      has_redos: self.undo_stack.has_redos(),
    };
    app_handle.emit(UndoAvailabilityPayload::EVENT_NAME, payload)
  }

  pub fn send_modeline_event(&self, app_handle: &tauri::AppHandle) -> tauri::Result<()> {
    let payload = ModelinePayload {
      modeline_text: self.modeline(),
    };
    app_handle.emit(ModelinePayload::EVENT_NAME, payload)
  }

  pub fn send_all_updates(&self, app_handle: &tauri::AppHandle, force_scroll_down: bool) -> tauri::Result<()> {
    self.send_refresh_stack_event(app_handle, force_scroll_down)?;
    self.send_undo_buttons_event(app_handle)?;
    self.send_modeline_event(app_handle)?;
    Ok(())
  }

  /// The modeline, which appears in teletype font at the bottom of
  /// the screen and indicates the current values of various
  /// user-specified flags.
  pub fn modeline(&self) -> String {
    ModelineBuilder::new()
      .append(self.display_settings().language_settings.preferred_radix)
      .append(boolean_flag("Inf", self.calculation_mode().has_infinity_flag()))
      .append(boolean_flag("Fr", self.calculation_mode().has_fractional_flag()))
      .append(LanguageModeValue::new(self.display_settings().base_language_mode.as_ref()))
      .append(boolean_flag("U", self.display_settings().language_settings.prefers_unicode_output))
      .append(boolean_flag("Gr", self.display_settings().is_graphics_enabled))
      .build()
  }

  pub fn display_settings(&self) -> &DisplaySettings {
    &self.undoable_state.display_settings
  }

  pub fn display_settings_mut(&mut self) -> &mut DisplaySettings {
    &mut self.undoable_state.display_settings
  }

  pub fn calculation_mode(&self) -> &CalculationMode {
    &self.undoable_state.calculation_mode
  }

  pub fn calculation_mode_mut(&mut self) -> &mut CalculationMode {
    &mut self.undoable_state.calculation_mode
  }

  pub fn variable_table(&self) -> &VarTable<Expr> {
    &self.undoable_state.variables
  }

  pub fn variable_table_mut(&mut self) -> &mut VarTable<Expr> {
    &mut self.undoable_state.variables
  }

  pub fn main_stack(&self) -> &Stack<Expr> {
    &self.undoable_state.main_stack
  }

  /// Returns the main stack as a mutable reference, without any undo
  /// semantics. The user of this function is then responsible for
  /// managing the undo stack themselves.
  ///
  /// Consider using
  /// [`main_stack_mut`](ApplicationState::main_stack_mut) instead,
  /// which provides the undo capabilities automatically.
  pub fn main_stack_mut_raw(&mut self) -> &mut Stack<Expr> {
    &mut self.undoable_state.main_stack
  }

  pub fn main_stack_mut(&mut self) -> DelegatingStack<'_, Stack<Expr>, UndoingDelegate<'_>> {
    DelegatingStack::new(
      self.undoable_state.main_stack_mut(),
      UndoingDelegate::new(&mut self.undo_stack),
    )
  }

  pub fn into_main_stack(self) -> Stack<Expr> {
    self.undoable_state.main_stack
  }

  pub fn undo_stack(&self) -> &UndoStack<UndoableState> {
    &self.undo_stack
  }

  pub fn undo_stack_mut(&mut self) -> &mut UndoStack<UndoableState> {
    &mut self.undo_stack
  }

  pub fn undo(&mut self) -> Result<(), UndoError> {
    self.undo_stack.undo(&mut self.undoable_state)
  }

  pub fn redo(&mut self) -> Result<(), UndoError> {
    self.undo_stack.redo(&mut self.undoable_state)
  }
}

impl UndoableState {
  pub fn main_stack(&self) -> &Stack<Expr> {
    &self.main_stack
  }

  pub fn main_stack_mut(&mut self) -> &mut Stack<Expr> {
    &mut self.main_stack
  }

  pub fn display_settings(&self) -> &DisplaySettings {
    &self.display_settings
  }

  pub fn display_settings_mut(&mut self) -> &mut DisplaySettings {
    &mut self.display_settings
  }

  pub fn calculation_mode(&self) -> &CalculationMode {
    &self.calculation_mode
  }

  pub fn calculation_mode_mut(&mut self) -> &mut CalculationMode {
    &mut self.calculation_mode
  }

  pub fn variable_table(&self) -> &VarTable<Expr> {
    &self.variables
  }

  pub fn variable_table_mut(&mut self) -> &mut VarTable<Expr> {
    &mut self.variables
  }
}

#[cfg(test)]
pub(crate) mod test_utils {
  use super::*;
  use crate::stack::test_utils::stack_of;

  /// Produces a default state, except that the state's `main_stack`
  /// is equal to `stack` (with the top of the stack being the last
  /// element in the vector).
  pub fn state_for_stack(stack: Vec<impl Into<Expr>>) -> ApplicationState {
    let mut state = ApplicationState::new();
    *state.main_stack_mut_raw() = stack_of(stack);
    state
  }
}
