
import { TAURI, defaultCommandOptions } from '../tauri_api.js';
import { TouchMode, TouchModeFactoryContext } from '../touch_mode.js';

import { Sortable, SortableStopEvent } from '@shopify/draggable';

export class DragTouchMode implements TouchMode {
  private valueStackDiv: HTMLElement;
  private sortable: Sortable | undefined;

  constructor(context: TouchModeFactoryContext) {
    this.valueStackDiv = context.valueStackDiv;
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
