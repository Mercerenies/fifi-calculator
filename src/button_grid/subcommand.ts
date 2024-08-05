
import { AbstractButtonManager, ButtonGrid, GridCell } from '../button_grid.js';
import { TAURI, SubcommandId } from '../tauri_api.js';
import { InputBoxManager } from '../input_box.js';
import { ButtonModifiers } from './modifier_delegate.js';

// The behavior of a button or grid cell when we try to use it as a
// subcommand. Valid behaviors are:
//
// * IsSubcommand: The button can be treated as a subcommand.
//
// * "pass": The button should fire like normal, even if we're
// inputting a subcommand.
//
// * "invalid": The button is NOT valid as a subcommand.
export type SubcommandBehavior = IsSubcommand | "pass" | "invalid";

export class IsSubcommand {
  readonly subcommand: SubcommandId;

  constructor(subcommand: SubcommandId) {
    this.subcommand = subcommand;
  }
}

export class SubcommandButtonManager implements AbstractButtonManager {
  private parent: AbstractButtonManager;
  private callback: (subcommand: SubcommandId) => Promise<void>;

  constructor(parent: AbstractButtonManager, callback: (subcommand: SubcommandId) => Promise<void>) {
    this.parent = parent;
    this.callback = callback;
  }

  get inputManager(): InputBoxManager {
    return this.parent.inputManager;
  }

  getModifiers(): ButtonModifiers {
    return this.parent.getModifiers();
  }

  setActiveGrid(grid: ButtonGrid): void {
    this.parent.setActiveGrid(grid);
  }

  resetState(): void {
    this.parent.resetState();
  }

  async invokeMathCommand(): Promise<void> {
    // If this gets called, we've accidentally invoked a subcommand as
    // a regular command, which is a bug in the program.
    throw new Error("Attempted to invoke regular command during subcommand input!");
  }

  setCurrentManager(manager: AbstractButtonManager): void {
    this.parent.setCurrentManager(manager);
  }

  async onClick(cell: GridCell): Promise<void> {
    try {
      const subcommand = cell.asSubcommand(this);
      if (subcommand === "pass") {
        await this.parent.onClick(cell);
      } else if (subcommand === "invalid") {
        TAURI.showError("Invalid subcommand");
      } else {
        await this.callback(subcommand.subcommand);
      }
    } finally {
      this.resetState();
      this.setCurrentManager(this.parent);
    }
  }

  async onEscape(): Promise<void> {
    this.setCurrentManager(this.parent);
  }
}
