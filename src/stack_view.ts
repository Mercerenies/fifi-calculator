
import { TAURI } from './tauri_api.js';
import { defaultCommandOptions } from './button_grid/modifier_delegate.js';

import { Sortable, SortableStopEvent } from '@shopify/draggable';

// Manager class for displaying the current value stack.
export class StackView {
  private valueStackDiv: HTMLElement;
  private sortable: Sortable | undefined;
  private currentStackSize: number = 0;

  constructor(valueStackDiv: HTMLElement) {
    this.valueStackDiv = valueStackDiv;
  }

  refreshStack(newStackHtml: string[]): void {
    if (this.sortable) {
      this.sortable.destroy();
      this.sortable = undefined;
    }

    this.currentStackSize = newStackHtml.length;

    const ol = document.createElement("ol");
    for (let i = 0; i < newStackHtml.length; i++) {
      const elem = newStackHtml[i];
      const li = document.createElement("li");
      li.className = 'value-stack-element';
      li.value = newStackHtml.length - i;
      li.dataset.stackIndex = String(newStackHtml.length - i - 1);
      li.innerHTML = elem;
      ol.appendChild(li);
    }
    const stack = this.valueStackDiv;
    stack.innerHTML = "";
    stack.appendChild(ol);

    this.sortable = new Sortable(ol, {
      draggable: 'li',
    });
    this.sortable.on('sortable:stop', (event) => this.onSortOrderUpdate(event));
  }

  scrollToBottom(): void {
    this.valueStackDiv.scrollTo({ top: this.valueStackDiv.scrollHeight });
  }

  private onSortOrderUpdate(event: SortableStopEvent): void {
    const { oldIndex, newIndex } = event;
    if ((oldIndex === undefined) || (newIndex === undefined)) {
      throw `Indices are undefined, got {oldIndex: ${oldIndex}, newIndex: ${newIndex}}`;
    }
    // For the stack shuffle command, we count from the top of the
    // stack, which is visually the bottom of the stack view.
    const srcIndex = this.currentStackSize - 1 - oldIndex;
    let destIndex = this.currentStackSize - 1 - newIndex;

    window.setTimeout(() => {
      TAURI.runMathCommand("move_stack_elem", [String(srcIndex), String(destIndex)], defaultCommandOptions());
    }, 1);
  }
}
