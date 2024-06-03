
import { InputBoxManager } from '../input_box.js';
import { FreeformInputMethod } from './freeform_input.js';

const tauri = window.__TAURI__.tauri;

const ALGEBRAIC_INPUT_PROMPT = "Alg:";

export async function algebraicInputToStack(manager: InputBoxManager, initialInput: string = ""): Promise<void> {
  const text = await manager.show(new FreeformInputMethod(ALGEBRAIC_INPUT_PROMPT), initialInput);
  if (text) {
    await tauri.invoke('submit_expr', { value: text });
  }
}
