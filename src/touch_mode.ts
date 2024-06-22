
import { StackUpdatedDelegate } from './stack_view.js';
import { TAURI } from './tauri_api.js';
import { defaultCommandOptions } from './button_grid/modifier_delegate.js';

import { Sortable, SortableStopEvent } from '@shopify/draggable';

// Manager for the "Touch Mode" radiobuttons which control what
// happens when you click/drag a stack element.
export class TouchModeManager implements StackUpdatedDelegate {
  private radiobuttonsDiv: HTMLElement;
  private valueStackDiv: HTMLElement;
  private touchMode: TouchMode = NULL_TOUCH_MODE;

  constructor(args: TouchModeManagerArgs) {
    this.radiobuttonsDiv = args.radiobuttonsDiv;
    this.valueStackDiv = args.valueStackDiv;
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
        this.touchMode = touchModeFactory(this.valueStackDiv);
        this.updateTouchMode();
      });
      if ((inputElem as HTMLInputElement).checked) {
        this.touchMode = touchModeFactory(this.valueStackDiv);
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
}

export const TouchModeFactories: Record<string, (elem: HTMLElement) => TouchMode> = {
  DRAG: (elem) => new DragTouchMode(elem),
  VIEW: () => NULL_TOUCH_MODE, // TODO
  EDIT: () => NULL_TOUCH_MODE, // TODO
};

export interface TouchMode {
  initTouchMode(): void;
  uninitTouchMode(): void;
}

const NULL_TOUCH_MODE: TouchMode = {
  initTouchMode() {},
  uninitTouchMode() {},
}

export class DragTouchMode implements TouchMode {
  private valueStackDiv: HTMLElement;
  private sortable: Sortable | undefined;

  constructor(valueStackDiv: HTMLElement) {
    this.valueStackDiv = valueStackDiv;
  }

  initTouchMode(): void {
    this.uninitTouchMode(); // un-init any existing touch mode

    const orderedList = this.valueStackDiv.querySelector('ol');
    if (!orderedList) {
      console.warn("Could not find <ol> in value stack div");
      return;
    }
    this.sortable = new Sortable(orderedList, {
      draggable: 'li',
    });
    this.sortable.on('sortable:stop', (event) => this.onSortOrderUpdate(event));
  }

  uninitTouchMode(): void {
    if (this.sortable) {
      this.sortable.destroy();
      this.sortable = undefined;
    }
  }

  private onSortOrderUpdate(event: SortableStopEvent): void {
    const { oldIndex, newIndex } = event;
    if ((oldIndex === undefined) || (newIndex === undefined)) {
      throw `Indices are undefined, got {oldIndex: ${oldIndex}, newIndex: ${newIndex}}`;
    }
    // For the stack shuffle command, we count from the top of the
    // stack, which is visually the bottom of the stack view.
    const currentStackSize = Number(this.valueStackDiv.dataset.stackLength);
    const srcIndex = currentStackSize - 1 - oldIndex;
    const destIndex = currentStackSize - 1 - newIndex;

    window.setTimeout(() => {
      TAURI.runMathCommand("move_stack_elem", [String(srcIndex), String(destIndex)], defaultCommandOptions());
    }, 1);
  }
}
