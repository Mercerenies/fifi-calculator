
// Manager class for the button grid that shows up on-screen and for
// keyboard shortcuts to said grid.

import { KeyEventInput, KeyResponse } from './keyboard.js';
import { ModifierDelegate, ButtonModifiers, modifiersToRustArgs } from './button_grid/modifier_delegate.js';

const tauri = window.__TAURI__.tauri;

// NOTE: This should be kept up to date with the .button-grid class in
// styles.css. If that value gets updated, update this as well!
const GRID_CELLS_PER_ROW = 5;

// This one doesn't appear in the CSS; it just determines how many
// nodes we generate.
const GRID_ROWS = 6;

export class ButtonGridManager {
  private domElement: HTMLElement;
  private rootGrid: ButtonGrid;
  private activeGrid: ButtonGrid;
  private modifierDelegate: ModifierDelegate;

  constructor(domElement: HTMLElement, initialGrid: ButtonGrid, modifierDelegate: ModifierDelegate) {
    this.domElement = domElement;
    this.rootGrid = initialGrid;
    this.activeGrid = initialGrid;
    this.modifierDelegate = modifierDelegate;
    this.setActiveGrid(initialGrid); // Initialize the grid
  }

  initListeners(): void {
    // No listeners to initialize for right now.
  }

  resetState(): void {
    this.setActiveGrid(this.rootGrid);
    this.resetModifiers();
  }

  resetModifiers(): void {
    this.modifierDelegate.resetModifiers();
  }

  getModifiers(): ButtonModifiers {
    return this.modifierDelegate.getModifiers();
  }

  setActiveGrid(grid: ButtonGrid): void {
    this.activeGrid = grid;
    this.loadHtml();
  }

  async invokeMathCommand(
    commandName: string,
    args: string[] = [],
    modifiersOverrides: Partial<ButtonModifiers> = {},
  ): Promise<void> {
    const modifiers = this.getModifiers();
    Object.assign(modifiers, modifiersOverrides);
    await tauri.invoke('run_math_command', {
      commandName,
      args,
      opts: modifiersToRustArgs(modifiers),
    });
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

  async onKeyDown(input: KeyEventInput): Promise<KeyResponse> {
    const responseFromDelegate = await this.modifierDelegate.onKeyDown(input);
    if (responseFromDelegate == KeyResponse.BLOCK) {
      // Delegate handled the event, so don't propagate to the
      // buttons.
      return KeyResponse.BLOCK;
    }
    const button = this.activeGrid.getKeyMappingTable()[input.toEmacsSyntax()];
    if (button !== undefined) {
      input.event.preventDefault();
      await button.fire(this);
      return KeyResponse.BLOCK;
    } else {
      return await this.activeGrid.onUnhandledKey(input, this);
    }
  }
}

export abstract class ButtonGrid {
  // This field is lazy-initialized on the first call to
  // getKeyMappingTable() and stored after that fact.
  private buttonsByKey: Record<string, GridCell> | null = null;

  // Should be at most a GRID_ROWS * GRID_CELLS_BY_ROW array. If this
  // grid is smaller than that size, the missing elements will be
  // filled in with Spacer objects.
  abstract get rows(): readonly (readonly GridCell[])[];

  /* eslint-disable-next-line @typescript-eslint/no-unused-vars */
  onUnhandledKey(input: KeyEventInput, manager: ButtonGridManager): Promise<KeyResponse> {
    // Default implementation is empty.
    return Promise.resolve(KeyResponse.PASS);
  }

  getAllCellKeys(): string[] {
    const keys = [];
    for (const row of this.rows) {
      for (const button of row) {
        if (button.keyboardShortcut !== null) {
          keys.push(button.keyboardShortcut);
        }
      }
    }
    return keys;
  }

  // By default, this method is equivalent to getAllCellKeys(). But
  // getAllKeys() can be overridden by subclasses to include keys
  // which delegate to other grids via onUnhandledKey().
  getAllKeys(): string[] {
    return this.getAllCellKeys();
  }

  getKeyMappingTable(): Record<string, GridCell> {
    if (this.buttonsByKey !== null) {
      return this.buttonsByKey;
    }
    this.buttonsByKey = {};
    for (const row of this.rows) {
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
    return this.buttonsByKey;
  }
}

export interface GridCell {
  readonly keyboardShortcut: string | null;

  getHTML(manager: ButtonGridManager): HTMLElement;
  fire(manager: ButtonGridManager): Promise<void>;
}

// Empty grid cell.
export class Spacer implements GridCell {
  readonly keyboardShortcut: string | null = null;

  getHTML(): HTMLElement {
    return document.createElement("div");
  }

  fire(): Promise<void> {
    // No action.
    return Promise.resolve();
  }
}
