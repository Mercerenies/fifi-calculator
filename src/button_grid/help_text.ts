
import { AbstractButtonManager, ButtonGrid, GridCell } from '../button_grid.js';
import { TAURI, SubcommandId } from '../tauri_api.js';
import { InputBoxManager } from '../input_box.js';
import { ButtonModifiers } from './modifier_delegate.js';

// TODO Visual indicator when this mode is active (some translucent
// gray texture over the screen or something?)

// Button manager for the mode in which the user has clicked the
// "help" button. The next button clicked will show help text instead
// of executing.
export class HelpModeButtonManager implements AbstractButtonManager {
  private parent: AbstractButtonManager;
  private callback: (cell: GridCell) => Promise<HelpModeResponse>;
  private cancelCallback: () => Promise<void>;

  readonly labelHTML: string = "Help Mode";

  constructor(
    parent: AbstractButtonManager,
    callback: (cell: GridCell) => Promise<HelpModeResponse>,
    opts: Partial<HelpModeButtonManagerOpts> = {},
  ) {
    this.parent = parent;
    this.callback = callback;
    this.cancelCallback = opts.cancelCallback ?? (() => Promise.resolve());
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
    // Help Mode manager should never *actually* invoke any math
    // commands.
    throw new Error("Attempted to invoke regular command during subcommand input!");
  }

  getCurrentManager(): AbstractButtonManager {
    return this.parent.getCurrentManager();
  }

  setCurrentManager(manager: AbstractButtonManager): void {
    this.parent.setCurrentManager(manager);
  }

  async onClick(cell: GridCell): Promise<void> {
    try {
      await this.callback(cell);
      this.setCurrentManager(this.parent);
    } finally {
      this.resetState();
    }
  }

  async onEscape(): Promise<void> {
    this.setCurrentManager(this.parent);
    await this.cancelCallback();
  }
}

export interface HelpModeButtonManagerOpts {
  cancelCallback: () => Promise<void>,
}

export type HelpModeResponse = "success" | "pass";
