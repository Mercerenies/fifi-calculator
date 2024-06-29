
import { InputBoxManager } from '../input_box.js';
import { TouchModeFactoryContext } from '../touch_mode.js';
import { ClickableTouchMode } from './clickable.js';
import { editStackFrame } from '../input_box/algebraic_input.js';

export class EditTouchMode extends ClickableTouchMode {
  private inputManager: InputBoxManager;

  constructor(context: TouchModeFactoryContext) {
    super(context);
    this.inputManager = context.uiManager.inputManager;
  }

  onClick(elem: HTMLElement): void {
    const frame = Number((elem as HTMLElement).dataset.stackIndex);
    editStackFrame(this.inputManager, frame, { isMouseInteraction: true });
  }
}
