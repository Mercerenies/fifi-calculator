
import { TAURI } from '../tauri_api.js';
import { defaultCommandOptions } from '../button_grid/modifier_delegate.js';
import { InputBoxManager } from '../input_box.js';
import { TouchModeFactoryContext } from '../touch_mode.js';
import { ClickableTouchMode } from './clickable.js';
import { FreeformInputMethod } from '../input_box/freeform_input.js';

const EDIT_INPUT_PROMPT = "Edit:";

export class EditTouchMode extends ClickableTouchMode {
  private inputManager: InputBoxManager;

  constructor(context: TouchModeFactoryContext) {
    super(context);
    this.inputManager = context.uiManager.inputManager;
  }

  onClick(elem: HTMLElement): void {
    const frame = Number((elem as HTMLElement).dataset.stackIndex);
    editStackFrame(this.inputManager, frame);
  }
}

async function editStackFrame(manager: InputBoxManager, index: number): Promise<void> {
  const isValid = await TAURI.validateStackSize(index + 1);
  if (!isValid) {
    return;
  }

  const currentStackValue = await TAURI.getEditableStackElem(index);
  const text = await manager.show(new FreeformInputMethod(EDIT_INPUT_PROMPT), currentStackValue);
  if (text) {
    await TAURI.runMathCommand("replace_stack_elem", [String(index), text], defaultCommandOptions());
  }
}
