
import * as os from '@tauri-apps/api/os';
import { invoke } from '@tauri-apps/api/tauri';
import { listen, EventCallback, UnlistenFn } from '@tauri-apps/api/event';

class TauriApi {
  osType(): Promise<os.OsType> {
    return os.type()
  }

  runMathCommand(commandName: string, args: string[], opts: CommandOptions): Promise<void> {
    return invoke('run_math_command', { commandName, args, opts });
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

  listen(event: 'refresh-stack', callback: EventCallback<RefreshStackPayload>): Promise<UnlistenFn>;
  listen(event: 'refresh-undo-availability', callback: EventCallback<UndoAvailabilityPayload>): Promise<UnlistenFn>;
  listen(event: 'show-error', callback: EventCallback<ShowErrorPayload>): Promise<UnlistenFn>;
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

export interface ShowErrorPayload {
  errorMessage: string;
}
