
import { InputBoxManager } from '../input_box.js';
import { FreeformInputMethod } from './freeform_input.js';
import { TAURI, Validator, defaultCommandOptions } from '../tauri_api.js';

export const STRING_INPUT_PROMPT = "Str:";

export async function stringInputToStack(manager: InputBoxManager, initialInput: string = ""): Promise<void> {
  const text = await manager.show(new FreeformInputMethod(STRING_INPUT_PROMPT), initialInput);
  if (text) {
    await TAURI.runMathCommand('push_string', [text], defaultCommandOptions());
  }
}
