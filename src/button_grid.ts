
// Manager class for the button grid that shows up on-screen and for
// keyboard shortcuts to said grid.

import { InputBoxManager, NumericalInputMethod } from './input_box.js';

const { invoke } = window.__TAURI__.tauri;

export class ButtonGridManager {
  private activeGrid: ButtonGrid;
  private buttonsByKey: {[key: string]: Button} = {};

  constructor(initialGrid: ButtonGrid) {
    this.activeGrid = initialGrid;
    this.loadButtons();
  }

  private loadButtons() {
    this.buttonsByKey = {};
    for (const row of this.activeGrid.rows) {
      for (const button of row) {
        const keyboardShortcut = button.keyboardShortcut;
        if (keyboardShortcut !== null) {
          if (keyboardShortcut in this.buttonsByKey) {
            console.warn("Duplicate keyboard shortcut in grid:", keyboardShortcut);
          }
          this.buttonsByKey[keyboardShortcut] = button;
        }
      }
    }
  }

  async onKeyDown(event: KeyboardEvent): Promise<void> {
    const button = this.buttonsByKey[event.key];
    if (button !== undefined) {
      event.preventDefault();
      await button.fire(this);
    } else {
      await this.activeGrid.onUnhandledKey(event);
    }
  }
}

export interface ButtonGrid {
  readonly rows: ReadonlyArray<ReadonlyArray<Button>>;

  onUnhandledKey(event: KeyboardEvent): Promise<void>;
}

export interface Button {
  readonly keyboardShortcut: string | null;
  fire(manager: ButtonGridManager): Promise<void>;
}

export class MainButtonGrid implements ButtonGrid {
  private static NUMERICAL_INPUT_START_KEYS = new Set([
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", ".", "e", "_",
  ]);

  readonly rows = [
    [new DispatchButton("+", "+")],
    [new DispatchButton("-", "-")],
    [new DispatchButton("*", "*")],
    [new DispatchButton("/", "/")],
    [new DispatchButton("pop", "Backspace")],
    [new DispatchButton("swap", "Tab")],
  ];

  private inputManager: InputBoxManager;

  constructor(inputManager: InputBoxManager) {
    this.inputManager = inputManager;
  }

  async onUnhandledKey(event: KeyboardEvent): Promise<void> {
    if (MainButtonGrid.NUMERICAL_INPUT_START_KEYS.has(event.key)) {
      // Start numerical input
      event.preventDefault();
      this.inputManager.show(new NumericalInputMethod(), this.translateInitialInput(event.key));
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

export class DispatchButton implements Button {
  readonly commandName: string;
  readonly keyboardShortcut: string | null;

  constructor(commandName: string, keyboardShortcut: string | null) {
    this.commandName = commandName;
    this.keyboardShortcut = keyboardShortcut;
  }

  fire(manager: ButtonGridManager): Promise<void> {
    return invoke('math_command', { commandName: this.commandName });
  }
}
