
import { ButtonGridManager, ButtonGrid, GridCell } from "../button_grid.js";
import { ButtonModifiers } from './modifier_delegate.js';
import { backButton, Button } from './button.js';
import { variableNameInput } from '../input_box/algebraic_input.js';
import { InputBoxManager } from '../input_box.js';
import { TAURI } from '../tauri_api.js';

export class StorageButtonGrid extends ButtonGrid {
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
        new VariableStoreButton(":=", "t", this.inputManager, false),
        new VariableStoreButton(":=<sup>K</sup>", "s", this.inputManager, true),
      ],
      [
        new VariableUnbindButton(this.inputManager),
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

export class VariableStoreButton extends Button {
  private inputManager: InputBoxManager;
  private overrideKeepModifier: boolean;

  constructor(label: string | HTMLElement, key: string | null, inputManager: InputBoxManager, overrideKeepModifier: boolean = false) {
    super(label, key);
    this.inputManager = inputManager;
    this.overrideKeepModifier = overrideKeepModifier;
  }

  async fire(manager: ButtonGridManager): Promise<void> {
    // Fire-and-forget a new promise that gets user input, so we don't
    // hold up the existing input.
    this.readAndStore(manager);
  }

  private async readAndStore(manager: ButtonGridManager): Promise<void> {
    try {
      const isValid = await TAURI.validateStackSize(1);
      if (!isValid) {
        return;
      }
      const variableName = await variableNameInput(this.inputManager);
      if (!variableName) {
        return;
      }
      await manager.invokeMathCommand('store_var', [variableName], this.modifiersOverride());
    } finally {
      manager.resetState();
    }
  }

  private modifiersOverride(): Partial<ButtonModifiers> {
    if (this.overrideKeepModifier) {
      return { keepModifier: true };
    } else {
      return {};
    }
  }
}

export class VariableUnbindButton extends Button {
  private inputManager: InputBoxManager;

  constructor(inputManager: InputBoxManager) {
    super("<math><mo lspace='0' rspace='0'>↚</mo></math>", "u");
    this.inputManager = inputManager;
  }

  async fire(manager: ButtonGridManager): Promise<void> {
    // Fire-and-forget a new promise that gets user input, so we don't
    // hold up the existing input.
    this.readAndStore(manager);
  }

  private async readAndStore(manager: ButtonGridManager): Promise<void> {
    try {
      const variableName = await variableNameInput(this.inputManager);
      if (!variableName) {
        return;
      }
      await manager.invokeMathCommand('unbind_var', [variableName]);
    } finally {
      manager.resetState();
    }
  }
}
