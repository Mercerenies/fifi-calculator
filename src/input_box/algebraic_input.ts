
import { InputBoxManager } from '../input_box.js';
import { FreeformInputMethod } from './freeform_input.js';
import { TAURI, Validator, defaultCommandOptions } from '../tauri_api.js';

export const ALGEBRAIC_INPUT_PROMPT = "Alg:";
export const VARIABLE_NAME_INPUT_PROMPT = "Var:";

export async function algebraicInputToStack(manager: InputBoxManager, initialInput: string = ""): Promise<void> {
  const text = await manager.show(new FreeformInputMethod(ALGEBRAIC_INPUT_PROMPT), initialInput);
  if (text) {
    await TAURI.runMathCommand('push_expr', [text], defaultCommandOptions());
  }
}

export async function variableNameInput(manager: InputBoxManager, initialInput: string = ""): Promise<string | undefined> {
  const text = await manager.show(new FreeformInputMethod(VARIABLE_NAME_INPUT_PROMPT), initialInput);
  if (!text) {
    return undefined;
  }
  const isValid = await TAURI.validateValue(text, Validator.VARIABLE);
  if (isValid) {
    return text;
  } else {
    return undefined;
  }
}
