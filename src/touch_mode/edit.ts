
import { TAURI } from '../tauri_api.js';
import { defaultCommandOptions } from '../button_grid/modifier_delegate.js';
import { InputBoxManager } from '../input_box.js';
import { TouchMode, TouchModeFactoryContext } from '../touch_mode.js';
import { FreeformInputMethod } from '../input_box/freeform_input.js';

const EDIT_INPUT_PROMPT = "Edit:";

export class EditTouchMode implements TouchMode {
  private valueStackDiv: HTMLElement;
  private inputManager: InputBoxManager;
  private unregisterFunctions: (() => void)[] = [];

  constructor(context: TouchModeFactoryContext) {
    this.valueStackDiv = context.valueStackDiv;
    this.inputManager = context.inputManager;
  }

  initTouchMode(): void {
    this.uninitTouchMode();

    const stackElems = this.valueStackDiv.querySelectorAll('li.value-stack-element');
    for (const elem of stackElems) {
      const fn = () => {
        const frame = Number((elem as HTMLElement).dataset.stackIndex);
        editStackFrame(this.inputManager, frame);
      };
      elem.addEventListener('click', fn);
      this.unregisterFunctions.push(() => elem.removeEventListener('click', fn));
    }
  }

  uninitTouchMode(): void {
    for (const fn of this.unregisterFunctions) {
      fn();
    }
    this.unregisterFunctions = [];
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
