
// Manager class for the button grid that shows up on-screen and for
// keyboard shortcuts to said grid.

import { InputBoxManager, NumericalInputMethod } from './input_box.js';

const { invoke } = window.__TAURI__.tauri;

// NOTE: This should be kept up to date with the .button-grid class in
// styles.css. If that value gets updated, update this as well!
const GRID_CELLS_PER_ROW = 5;

// This one doesn't appear in the CSS; it just determines how many
// nodes we generate.
const GRID_ROWS = 6;

export class ButtonGridManager {
  private domElement: HTMLElement;
  private activeGrid: ButtonGrid;
  private buttonsByKey: {[key: string]: GridCell} = {};

  constructor(domElement: HTMLElement, initialGrid: ButtonGrid) {
    this.domElement = domElement;
    this.activeGrid = initialGrid;
    this.setActiveGrid(initialGrid); // Initialize the grid
  }

  setActiveGrid(grid: ButtonGrid): void {
    this.activeGrid = grid;
    this.loadButtonShortcuts();
    this.loadHtml();
  }

  private loadButtonShortcuts(): void {
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

  private loadHtml(): void {
    this.domElement.innerHTML = "";
    const gridDiv = document.createElement("div");
    gridDiv.classList.add("button-grid");
    for (let y = 0; y < GRID_ROWS; y++) {
      const row = this.activeGrid.rows[y] ?? [];
      for (let x = 0; x < GRID_CELLS_PER_ROW; x++) {
        const gridCell = row[x] ?? new Spacer();
        gridDiv.appendChild(gridCell.getHTML(this));
      }
    }
    this.domElement.appendChild(gridDiv);
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
  readonly rows: ReadonlyArray<ReadonlyArray<GridCell>>;

  onUnhandledKey(event: KeyboardEvent): Promise<void>;
}

export interface GridCell {
  readonly keyboardShortcut: string | null;

  getHTML(manager: ButtonGridManager): HTMLElement;
  fire(manager: ButtonGridManager): Promise<void>;
}

export class MainButtonGrid implements ButtonGrid {
  // TODO Escape on main button grid should close notification?

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

// Empty grid cell.
export class Spacer implements GridCell {
  readonly keyboardShortcut: string | null = null;

  getHTML(manager: ButtonGridManager): HTMLElement {
    return document.createElement("div");
  }

  fire(manager: ButtonGridManager): Promise<void> {
    // No action.
    return Promise.resolve();
  }
}

export abstract class Button implements GridCell {
  readonly label: string;
  readonly keyboardShortcut: string | null;

  constructor(label: string, keyboardShortcut: string | null) {
    this.label = label;
    this.keyboardShortcut = keyboardShortcut;
  }

  getHTML(manager: ButtonGridManager): HTMLElement {
    const button = document.createElement("button");
    button.innerHTML = this.label;
    button.addEventListener("click", () => this.fire(manager));
    return button;
  }

  abstract fire(manager: ButtonGridManager): Promise<void>;
}

export class DispatchButton extends Button {
  readonly commandName: string;

  constructor(label: string, commandName: string, keyboardShortcut: string | null) {
    super(label, keyboardShortcut);
    this.commandName = commandName;
  }

  fire(manager: ButtonGridManager): Promise<void> {
    return invoke('math_command', { commandName: this.commandName });
  }
}
