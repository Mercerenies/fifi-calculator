
import { ButtonGrid, GridCell } from "../button_grid.js";
import { NumericalInputButton, DispatchButton } from './button.js';
import { InputBoxManager } from '../input_box.js';
import { NumericalInputMethod } from '../input_box/numerical_input.js';
import { KeyEventInput } from '../keyboard.js';
import { svg } from '../util.js';

export interface Hideable {
  hide(): void;
}

function discardSvg(): HTMLElement {
  return svg('assets/discard.svg', {alt: "pop"});
}

function swapSvg(): HTMLElement {
  return svg('assets/swap.svg', {alt: "swap"});
}

function dupSvg(): HTMLElement {
  return svg('assets/duplicate.svg', {alt: "dup"});
}

export class MainButtonGrid implements ButtonGrid {
  private static NUMERICAL_INPUT_START_KEYS = new Set([
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", ".", "e", "_",
  ]);

  readonly rows: readonly (readonly GridCell[])[];

  private inputManager: InputBoxManager;
  private onEscapeDismissable: Hideable;

  constructor(inputManager: InputBoxManager, onEscapeDismissable: Hideable) {
    this.inputManager = inputManager;
    this.onEscapeDismissable = onEscapeDismissable;
    this.rows = this.initRows();
  }

  private initRows(): GridCell[][] {
    return [
      [new DispatchButton("+", "+", "+")],
      [new DispatchButton("-", "-", "-")],
      [new DispatchButton("&times;", "*", "*")],
      [
        new DispatchButton("&divide;", "/", "/"),
        new DispatchButton("%", "%", "%"),
      ],
      [
        new DispatchButton(discardSvg(), "pop", "Backspace"),
        new DispatchButton(swapSvg(), "swap", "Tab"),
        new DispatchButton(dupSvg(), "dup", "Enter"),
        new NumericalInputButton(this.inputManager),
      ],
    ];
  }

  async onUnhandledKey(input: KeyEventInput): Promise<void> {
    const key = input.toEmacsSyntax();
    if (MainButtonGrid.NUMERICAL_INPUT_START_KEYS.has(key)) {
      // Start numerical input
      input.event.preventDefault();
      this.inputManager.show(new NumericalInputMethod(), this.translateInitialInput(key));
    } else if (key === "Escape") {
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
