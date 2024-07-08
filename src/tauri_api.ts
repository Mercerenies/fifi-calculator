
import { SerializedGraphicsPayload, GraphicsResponse } from './tauri_api/graphics.js';
import { modifiersToRustArgs, defaultModifiers } from './button_grid/modifier_delegate.js';

import * as os from '@tauri-apps/plugin-os';
import { invoke } from '@tauri-apps/api/core';
import { listen, EventCallback, UnlistenFn } from '@tauri-apps/api/event';

class TauriApi {
  osType(): Promise<os.OsType> {
    return os.type()
  }

  runMathCommand(commandName: string, args: string[], opts: CommandOptions): Promise<void> {
    return invoke('run_math_command', { commandName, args, opts });
  }

  renderGraphics(payload: SerializedGraphicsPayload): Promise<GraphicsResponse | null> {
    return invoke('render_graphics', { payload });
  }

  getEditableStackElem(stackIndex: number): Promise<string> {
    return invoke('get_editable_stack_elem', { stackIndex });
  }

  performUndoAction(direction: UndoDirection): Promise<void> {
    return invoke('perform_undo_action', { direction });
  }

  validateStackSize(expected: number): Promise<boolean> {
    return invoke('validate_stack_size', { expected });
  }

  validateValue(value: string, validator: Validator): Promise<boolean> {
    return invoke('validate_value', { value, validator });
  }

  queryStack(query: StackQuery): Promise<boolean> {
    return invoke('query_stack', { query });
  }

  listen(event: 'refresh-stack', callback: EventCallback<RefreshStackPayload>): Promise<UnlistenFn>;
  listen(event: 'refresh-undo-availability', callback: EventCallback<UndoAvailabilityPayload>): Promise<UnlistenFn>;
  listen(event: 'refresh-modeline', callback: EventCallback<ModelinePayload>): Promise<UnlistenFn>;
  listen(event: 'show-error', callback: EventCallback<ShowErrorPayload>): Promise<UnlistenFn>;
  /* eslint-disable-next-line @typescript-eslint/no-explicit-any */
  listen(event: string, callback: EventCallback<any>): Promise<UnlistenFn> {
    return listen(event, callback);
  }
}

export const TAURI = new TauriApi();

export enum UndoDirection {
  UNDO = "undo",
  REDO = "redo",
}

export enum Validator {
  VARIABLE = "variable",
  ALL_UNITS = "all_units",
  HAS_UNITS = "has_units",
}

export interface CommandOptions {
  argument: number | null,
  keepModifier: boolean,
}

export interface RefreshStackPayload {
  stack: string[];
  forceScrollDown: boolean;
}

export interface UndoAvailabilityPayload {
  hasUndos: boolean;
  hasRedos: boolean;
}

export interface ModelinePayload {
  modelineText: string;
}

export interface ShowErrorPayload {
  errorMessage: string;
}

export interface StackQuery {
  stack_index: number;
  query_type: StackQueryType;
}

export enum StackQueryType {
  HAS_UNITS = "has_units",
}

export function defaultCommandOptions(): CommandOptions {
  return modifiersToRustArgs(defaultModifiers());
}
