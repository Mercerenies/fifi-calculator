
import { StackUpdatedDelegate } from './stack_view.js';
import { InputBoxManager } from './input_box.js';
import { DragTouchMode } from './touch_mode/drag.js';
import { EditTouchMode } from './touch_mode/edit.js';
import { UiManager } from './ui_manager.js';

// Manager for the "Touch Mode" radiobuttons which control what
// happens when you click/drag a stack element.
export class TouchModeManager implements StackUpdatedDelegate {
  private radiobuttonsDiv: HTMLElement;
  private touchMode: TouchMode = NULL_TOUCH_MODE;
  readonly valueStackDiv: HTMLElement;
  readonly uiManager: UiManager;

  constructor(args: TouchModeManagerArgs) {
    this.radiobuttonsDiv = args.radiobuttonsDiv;
    this.valueStackDiv = args.valueStackDiv;
    this.uiManager = args.uiManager;
  }

  initListeners(): void {
    for (const inputElem of this.radiobuttonsDiv.querySelectorAll('input[type="radio"]')) {
      const touchModeData = (inputElem as HTMLElement).dataset.touchMode;
      if (touchModeData == undefined) {
        throw "No touchMode data on radio button";
      }
      const touchModeFactory = TouchModeFactories[touchModeData];
      if (touchModeFactory == undefined) {
        throw "Invalid touchMode data on radio button";
      }
      inputElem.addEventListener('change', () => {
        this.touchMode.uninitTouchMode();
        this.touchMode = touchModeFactory(this);
        this.updateTouchMode();
      });
      if ((inputElem as HTMLInputElement).checked) {
        this.touchMode = touchModeFactory(this);
        // NOTE: Don't call updateTouchMode, as the HTML may not have
        // been fully set up by this point. The first stack update
        // will trigger the init call.
      }
    }
  }

  private updateTouchMode(): void {
    this.touchMode.initTouchMode();
  }

  async onStackUpdated(): Promise<void> {
    this.updateTouchMode();
  }
}

export interface TouchModeManagerArgs {
  radiobuttonsDiv: HTMLElement;
  valueStackDiv: HTMLElement;
  uiManager: UiManager;
}

export interface TouchModeFactoryContext {
  valueStackDiv: HTMLElement;
  uiManager: UiManager;
}

export const TouchModeFactories: Record<string, (ctx: TouchModeFactoryContext) => TouchMode> = {
  DRAG: (ctx) => new DragTouchMode(ctx),
  VIEW: () => NULL_TOUCH_MODE, // TODO /////
  EDIT: (ctx) => new EditTouchMode(ctx),
};

export interface TouchMode {
  initTouchMode(): void;
  uninitTouchMode(): void;
}

const NULL_TOUCH_MODE: TouchMode = {
  initTouchMode() {},
  uninitTouchMode() {},
}
