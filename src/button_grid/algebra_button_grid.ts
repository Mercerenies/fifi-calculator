
import { AbstractButtonManager, ButtonGrid, GridCell } from "../button_grid.js";
import { backButton, Button } from './button.js';
import { variableNameInput } from '../input_box/algebraic_input.js';
import { FreeformInputMethod } from '../input_box/freeform_input.js';
import { TAURI } from '../tauri_api.js';

export class AlgebraButtonGrid extends ButtonGrid {
  readonly rows: readonly (readonly GridCell[])[];

  private rootGrid: ButtonGrid;

  constructor(rootGrid: ButtonGrid) {
    super();
    this.rootGrid = rootGrid;
    this.rows = this.initRows();
  }

  private initRows(): GridCell[][] {
    return [
      [
        new VariableSubstituteButton(),
      ],
      [
        new FindRootButton(),
        new DerivativeButton(),
      ],
      [],
      [],
      [],
      [
        backButton(this.rootGrid),
      ],
    ];
  }
}

export class VariableSubstituteButton extends Button {
  constructor() {
    super("/.", "b");
  }

  async fire(manager: AbstractButtonManager): Promise<void> {
    // Fire-and-forget a new promise that gets user input, so we don't
    // hold up the existing input.
    this.readAndSubstitute(manager);
  }

  private async readAndSubstitute(manager: AbstractButtonManager): Promise<void> {
    try {
      const isValid = await TAURI.validateStackSize(1);
      if (!isValid) {
        return;
      }
      const variableName = await variableNameInput(manager.inputManager);
      if (!variableName) {
        return;
      }
      const newValue = await manager.inputManager.show(new FreeformInputMethod("Subst:"));
      if (!newValue) {
        return;
      }
      await manager.invokeMathCommand('manual_substitute', [variableName, newValue]);
    } finally {
      manager.resetState();
    }
  }
}

// TODO: Common superclass for buttons which expect one variable as
// input and call a command with it.
export class FindRootButton extends Button {

  constructor() {
    super("=0", "R");
  }

  async fire(manager: AbstractButtonManager): Promise<void> {
    // Fire-and-forget a new promise that gets user input, so we don't
    // hold up the existing input.
    this.readAndApply(manager);
  }

  private async readAndApply(manager: AbstractButtonManager): Promise<void> {
    try {
      const isValid = await TAURI.validateStackSize(2);
      if (!isValid) {
        return;
      }
      const variableName = await variableNameInput(manager.inputManager);
      if (!variableName) {
        return;
      }
      await manager.invokeMathCommand('find_root', [variableName]);
    } finally {
      manager.resetState();
    }
  }
}

export class DerivativeButton extends Button {

  constructor() {
    super("<span class='mathy-text'>dx</span>", "d");
  }

  async fire(manager: AbstractButtonManager): Promise<void> {
    // Fire-and-forget a new promise that gets user input, so we don't
    // hold up the existing input.
    this.readAndApply(manager);
  }

  private async readAndApply(manager: AbstractButtonManager): Promise<void> {
    try {
      const isValid = await TAURI.validateStackSize(1);
      if (!isValid) {
        return;
      }
      const variableName = await variableNameInput(manager.inputManager);
      if (!variableName) {
        return;
      }
      await manager.invokeMathCommand('deriv', [variableName]);
    } finally {
      manager.resetState();
    }
  }
}
