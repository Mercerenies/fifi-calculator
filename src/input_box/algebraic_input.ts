
import { InputBoxManager } from '../input_box.js';
import { FreeformInputMethod } from './freeform_input.js';
import { TAURI, Validator, defaultCommandOptions, CommandOptions } from '../tauri_api.js';

export const ALGEBRAIC_INPUT_PROMPT = "Alg:";
export const VARIABLE_NAME_INPUT_PROMPT = "Var:";
export const EDIT_INPUT_PROMPT = "Edit:";

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

export async function editStackFrame(
  manager: InputBoxManager,
  stackIndex: number,
  args: EditStackFrameArgs,
): Promise<void> {
  const isValid = await TAURI.validateStackSize(stackIndex + 1);
  if (!isValid) {
    return;
  }

  const currentStackValue = await TAURI.getEditableStackElem(stackIndex);
  const prompt = args.inputPrompt ?? EDIT_INPUT_PROMPT;
  const text = await manager.show(new FreeformInputMethod(prompt), currentStackValue);
  if (text) {
    const command = args.isMouseInteraction ? "mouse_replace_stack_elem" : "replace_stack_elem";
    const options = Object.assign(defaultCommandOptions(), args.commandOptionsOverride ?? {});
    await TAURI.runMathCommand(command, [String(stackIndex), text], options);
  }
}

export interface EditStackFrameArgs {
  isMouseInteraction: boolean;
  inputPrompt?: string;
  commandOptionsOverride?: Partial<CommandOptions>;
}
