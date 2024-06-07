
import { ButtonGridManager, ButtonGrid, GridCell } from "../button_grid.js";
import { KeyResponse } from '../keyboard.js';
import { backButton, Button } from './button.js';
import { variableNameInput } from '../input_box/algebraic_input.js';
import { FreeformInputMethod } from '../input_box/freeform_input.js';
import { InputBoxManager } from '../input_box.js';

const tauri = window.__TAURI__.tauri;

export class AlgebraButtonGrid implements ButtonGrid {
  readonly rows: readonly (readonly GridCell[])[];

  private rootGrid: ButtonGrid;
  private inputManager: InputBoxManager;

  constructor(rootGrid: ButtonGrid, inputManager: InputBoxManager) {
    this.rootGrid = rootGrid;
    this.inputManager = inputManager;
    this.rows = this.initRows();
  }

  private initRows(): GridCell[][] {
    return [
      [
        new VariableSubstituteButton(this.inputManager),
      ],
      [],
      [],
      [],
      [],
      [
        backButton(this.rootGrid),
      ],
    ];
  }

  onUnhandledKey(): Promise<KeyResponse> {
    return Promise.resolve(KeyResponse.PASS);
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
    manager.resetState();
  }

  private async readAndSubstitute(manager: ButtonGridManager): Promise<void> {
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
  }
}
