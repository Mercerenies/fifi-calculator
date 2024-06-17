
const __TAURI__ = window.__TAURI__;

class TauriApi {
  osType(): Promise<OsType> {
    return __TAURI__.os.type()
  }

  runMathCommand(commandName: string, args: string[], opts: CommandOptions): Promise<void> {
    return __TAURI__.tauri.invoke('run_math_command', { commandName, args, opts });
  }

  performUndoAction(direction: UndoDirection): Promise<void> {
    return __TAURI__.tauri.invoke('perform_undo_action', { direction });
  }

  validateStackSize(expected: number): Promise<boolean> {
    return __TAURI__.tauri.invoke('validate_stack_size', { expected });
  }

  validateValue(value: string, validator: Validator): Promise<boolean> {
    return __TAURI__.tauri.invoke('validate_value', { value, validator });
  }

  listen(event: 'refresh-stack', callback: EventCallback<RefreshStackPayload>): Promise<UnlistenFunction>;
  listen(event: 'refresh-undo-availability', callback: EventCallback<UndoAvailabilityPayload>): Promise<UnlistenFunction>;
  listen(event: 'show-error', callback: EventCallback<ShowErrorPayload>): Promise<UnlistenFunction>;
  listen(event: any, callback: EventCallback<any>): Promise<UnlistenFunction> {
    return __TAURI__.event.listen(event, callback);
  }
}

export const TAURI = new TauriApi();

export type UndoDirection = "undo" | "redo";
