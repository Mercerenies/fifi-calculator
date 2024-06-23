
import { TouchMode, TouchModeFactoryContext } from '../touch_mode.js';

// TouchMode which simply registers an onClick listener on each
// element of the stack.
export abstract class ClickableTouchMode implements TouchMode {
  private valueStackDiv: HTMLElement;
  private unregisterFunctions: (() => void)[] = [];

  constructor(context: TouchModeFactoryContext) {
    this.valueStackDiv = context.valueStackDiv;
  }

  initTouchMode(): void {
    this.uninitTouchMode();

    const stackElems = this.valueStackDiv.querySelectorAll('li.value-stack-element');
    for (const elem of stackElems) {
      const fn = () => {
        this.onClick(elem as HTMLElement);
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

  abstract onClick(element: HTMLElement): void;
}
