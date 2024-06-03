
import { InputBoxManager } from '../input_box.js';
import { FreeformInputMethod } from './freeform_input.js';

const tauri = window.__TAURI__.tauri;

const ALGEBRAIC_INPUT_PROMPT = "Alg:";
const VARIABLE_NAME_INPUT_PROMPT = "Var:";

export async function algebraicInputToStack(manager: InputBoxManager, initialInput: string = ""): Promise<void> {
  const text = await manager.show(new FreeformInputMethod(ALGEBRAIC_INPUT_PROMPT), initialInput);
  if (text) {
    await tauri.invoke('submit_expr', { value: text });
  }
}

export async function variableNameInput(manager: InputBoxManager, initialInput: string = ""): Promise<string | undefined> {
  const text = await manager.show(new FreeformInputMethod(VARIABLE_NAME_INPUT_PROMPT), initialInput);
  if (!text) {
    return undefined;
  }
  const isValid = await tauri.invoke('validate_value', { value: text, validator: "variable" });
  if (isValid) {
    return text;
  } else {
    return undefined;
  }
}
