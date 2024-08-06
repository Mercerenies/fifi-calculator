
// Manager class for the button grid that shows up on-screen and for
// keyboard shortcuts to said grid.

import { KeyEventInput, KeyResponse } from './keyboard.js';
import { ModifierDelegate, ButtonModifiers, modifiersToRustArgs } from './button_grid/modifier_delegate.js';
import { SubcommandBehavior } from './button_grid/subcommand.js';
import { TAURI } from './tauri_api.js';
import { InputBoxManager } from './input_box.js';

// NOTE: This should be kept up to date with the .button-grid class in
// styles.css. If that value gets updated, update this as well!
const GRID_CELLS_PER_ROW = 5;

// This one doesn't appear in the CSS; it just determines how many
// nodes we generate.
const GRID_ROWS = 6;

export class ButtonGridManager implements AbstractButtonManager {
  private domElement: HTMLElement;
  private rootGrid: ButtonGrid;
  private activeGrid: ButtonGrid;
  private modifierDelegate: ModifierDelegate;
  private onEscapeDismissable: Hideable;
  private buttonGridLabel: HTMLElement;
  private managerFacade: ManagerFacade;

  readonly inputManager: InputBoxManager;
  readonly labelHTML: string = "&nbsp;";

  constructor(args: ButtonGridManagerArgs) {
    this.domElement = args.domElement;
    this.rootGrid = args.initialGrid;
    this.activeGrid = args.initialGrid;
    this.modifierDelegate = args.modifierDelegate;
    this.inputManager = args.inputManager;
    this.onEscapeDismissable = args.onEscapeDismissable;
    this.buttonGridLabel = args.buttonGridLabel;
    this.managerFacade = new ManagerFacade(this);
    this.setActiveGrid(args.initialGrid); // Initialize the grid
  }

  initListeners(): void {
    // No listeners to initialize for right now.
  }

  isRootGrid(): boolean {
    return this.activeGrid === this.rootGrid;
  }

  resetState(): void {
    this.setActiveGrid(this.rootGrid);
    this.resetModifiers();
  }

  setCurrentManager(manager: AbstractButtonManager): void {
    this.managerFacade.setCurrentManager(manager);
    this.buttonGridLabel.innerHTML = manager.labelHTML;
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
    await TAURI.runMathCommand(commandName, args, modifiersToRustArgs(modifiers));
  }

  private loadHtml(): void {
    this.domElement.innerHTML = "";
    const gridDiv = document.createElement("div");
    gridDiv.classList.add("button-grid");
    for (let y = 0; y < GRID_ROWS; y++) {
      const row = this.activeGrid.rows[y] ?? [];
      for (let x = 0; x < GRID_CELLS_PER_ROW; x++) {
        const gridCell = row[x] ?? new Spacer();
        gridDiv.appendChild(gridCell.getHTML(this.managerFacade));
      }
    }
    this.domElement.appendChild(gridDiv);
  }

  async onKeyDown(input: KeyEventInput): Promise<KeyResponse> {
    if (this.isRootGrid()) {
      const responseFromDelegate = await this.modifierDelegate.onKeyDown(input);
      if (responseFromDelegate == KeyResponse.BLOCK) {
        // Delegate handled the event, so don't propagate to the
        // buttons.
        return KeyResponse.BLOCK;
      }
    }
    const button = this.activeGrid.getKeyMappingTable()[input.toEmacsSyntax()];
    if (button !== undefined) {
      input.event.preventDefault();
      await this.managerFacade.onClick(button);
      return KeyResponse.BLOCK;
    } else {
      return await this.activeGrid.onUnhandledKey(input, this.managerFacade);
    }
  }

  async onClick(cell: GridCell): Promise<void> {
    cell.fire(this);
  }

  async onEscape(): Promise<void> {
    this.onEscapeDismissable.hide();
  }
}

// Private helper which delegates to a settable AbstractButtonManager.
// We have this extra layer of indirection so we can set up all of the
// HTML for the button grids one time (with one object as the closure)
// and have this object dispatch to the appropriate place for a
// command or subcommand, based on the state of the system.
class ManagerFacade implements AbstractButtonManager {
  private currentManager: AbstractButtonManager;

  constructor(manager: ButtonGridManager) {
    this.currentManager = manager;
  }

  get inputManager(): InputBoxManager {
    return this.currentManager.inputManager;
  }

  get labelHTML(): string {
    return this.currentManager.labelHTML;
  }

  getModifiers(): ButtonModifiers {
    return this.currentManager.getModifiers();
  }

  setActiveGrid(grid: ButtonGrid): void {
    this.currentManager.setActiveGrid(grid);
  }

  resetState(): void {
    this.currentManager.resetState();
  }

  invokeMathCommand(
    commandName: string,
    args?: string[],
    modifiersOverrides?: Partial<ButtonModifiers>,
  ): Promise<void> {
    return this.currentManager.invokeMathCommand(commandName, args, modifiersOverrides);
  }

  onClick(cell: GridCell): Promise<void> {
    return this.currentManager.onClick(cell);
  }

  onEscape(): Promise<void> {
    return this.currentManager.onEscape();
  }

  setCurrentManager(manager: AbstractButtonManager): void {
    this.currentManager = manager;
  }
}

export interface Hideable {
  hide(): void;
}

export interface ButtonGridManagerArgs {
  domElement: HTMLElement;
  initialGrid: ButtonGrid;
  modifierDelegate: ModifierDelegate;
  inputManager: InputBoxManager;
  onEscapeDismissable: Hideable;
  buttonGridLabel: HTMLElement;
}

export interface AbstractButtonManager {
  readonly inputManager: InputBoxManager;
  readonly labelHTML: string;

  getModifiers(): ButtonModifiers;
  setActiveGrid(grid: ButtonGrid): void;
  resetState(): void;
  invokeMathCommand(
    commandName: string,
    args?: string[],
    modifiersOverrides?: Partial<ButtonModifiers>,
  ): Promise<void>;

  onClick(cell: GridCell): Promise<void>;
  onEscape(): Promise<void>;
  setCurrentManager(manager: AbstractButtonManager): void;
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
  onUnhandledKey(input: KeyEventInput, manager: AbstractButtonManager): Promise<KeyResponse> {
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
    // TODO: Currently no subclasses actually override this. Do we even use it?
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

  getHTML(manager: AbstractButtonManager): HTMLElement;
  fire(manager: AbstractButtonManager): Promise<void>;
  asSubcommand(manager: AbstractButtonManager): SubcommandBehavior;
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

  asSubcommand(): SubcommandBehavior {
    // No action.
    return "pass";
  }
}
