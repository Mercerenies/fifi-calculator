
import { ButtonGridManager, ButtonGrid, GridCell } from "../button_grid.js";
import { backButton, Button } from './button.js';
import { variableNameInput } from '../input_box/algebraic_input.js';
import { FreeformInputMethod } from '../input_box/freeform_input.js';
import { InputBoxManager } from '../input_box.js';

const tauri = window.__TAURI__.tauri;

export class AlgebraButtonGrid extends ButtonGrid {
  readonly rows: readonly (readonly GridCell[])[];

  private rootGrid: ButtonGrid;
  private inputManager: InputBoxManager;

  constructor(rootGrid: ButtonGrid, inputManager: InputBoxManager) {
    super();
    this.rootGrid = rootGrid;
    this.inputManager = inputManager;
    this.rows = this.initRows();
  }

  private initRows(): GridCell[][] {
    return [
      [
        new VariableSubstituteButton(this.inputManager),
      ],
      [
        new FindRootButton(this.inputManager),
        new DerivativeButton(this.inputManager),
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
  private inputManager: InputBoxManager;

  constructor(inputManager: InputBoxManager) {
    super("/.", "b");
    this.inputManager = inputManager;
  }

  async fire(manager: ButtonGridManager): Promise<void> {
    // Fire-and-forget a new promise that gets user input, so we don't
    // hold up the existing input.
    this.readAndSubstitute(manager);
  }

  private async readAndSubstitute(manager: ButtonGridManager): Promise<void> {
    try {
      const isValid = await tauri.invoke('validate_stack_size', { expected: 1 });
      if (!isValid) {
        return;
      }
      const variableName = await variableNameInput(this.inputManager);
      if (!variableName) {
        return;
      }
      const newValue = await this.inputManager.show(new FreeformInputMethod("Subst"));
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
  private inputManager: InputBoxManager;

  constructor(inputManager: InputBoxManager) {
    super("=0", "R");
    this.inputManager = inputManager;
  }

  async fire(manager: ButtonGridManager): Promise<void> {
    // Fire-and-forget a new promise that gets user input, so we don't
    // hold up the existing input.
    this.readAndApply(manager);
  }

  private async readAndApply(manager: ButtonGridManager): Promise<void> {
    try {
      const isValid = await tauri.invoke('validate_stack_size', { expected: 2 });
      if (!isValid) {
        return;
      }
      const variableName = await variableNameInput(this.inputManager);
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
  private inputManager: InputBoxManager;

  constructor(inputManager: InputBoxManager) {
    super("<span class='mathy-text'>dx</span>", "d");
    this.inputManager = inputManager;
  }

  async fire(manager: ButtonGridManager): Promise<void> {
    // Fire-and-forget a new promise that gets user input, so we don't
    // hold up the existing input.
    this.readAndApply(manager);
  }

  private async readAndApply(manager: ButtonGridManager): Promise<void> {
    try {
      const isValid = await tauri.invoke('validate_stack_size', { expected: 1 });
      if (!isValid) {
        return;
      }
      const variableName = await variableNameInput(this.inputManager);
      if (!variableName) {
        return;
      }
      await manager.invokeMathCommand('deriv', [variableName]);
    } finally {
      manager.resetState();
    }
  }
}
