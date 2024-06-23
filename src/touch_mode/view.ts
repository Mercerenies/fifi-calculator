
import { UiManager } from '../ui_manager.js';
import { TouchModeFactoryContext } from '../touch_mode.js';
import { ClickableTouchMode } from './clickable.js';

export class ViewTouchMode extends ClickableTouchMode {
  private uiManager: UiManager;

  constructor(context: TouchModeFactoryContext) {
    super(context);
    this.uiManager = context.uiManager;
  }

  onClick(elem: HTMLElement): void {
    this.uiManager.showPopup(elem, undefined);
  }
}
