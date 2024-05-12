
import { ButtonGrid } from "../button_grid.js";
import { DispatchButton } from './button.js';
import { InputBoxManager } from '../input_box.js';
import { NumericalInputMethod } from '../input_box/numerical_input.js';

export interface Hideable {
  hide(): void;
}

export class MainButtonGrid implements ButtonGrid {
  private static NUMERICAL_INPUT_START_KEYS = new Set([
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", ".", "e", "_",
  ]);

  readonly rows = [
    [new DispatchButton("+", "+", "+")],
    [new DispatchButton("-", "-", "-")],
    [new DispatchButton("&times;", "*", "*")],
    [new DispatchButton("&divide;", "/", "/")],
    [
      new DispatchButton("p", "pop", "Backspace"), // TODO Better label
      new DispatchButton("s", "swap", "Tab"), // TODO Better label
    ],
  ];

  private inputManager: InputBoxManager;
  private onEscapeDismissable: Hideable;

  constructor(inputManager: InputBoxManager, onEscapeDismissable: Hideable) {
    this.inputManager = inputManager;
    this.onEscapeDismissable = onEscapeDismissable;
  }

  async onUnhandledKey(event: KeyboardEvent): Promise<void> {
    if (MainButtonGrid.NUMERICAL_INPUT_START_KEYS.has(event.key)) {
      // Start numerical input
      event.preventDefault();
      this.inputManager.show(new NumericalInputMethod(), this.translateInitialInput(event.key));
    } else if (event.key === "Escape") {
      this.onEscapeDismissable.hide();
    }
  }

  private translateInitialInput(key: string): string {
    switch (key) {
    case "e":
      return "1e";
    case "_":
      return "-";
    default:
      return key;
    }
  }
}
