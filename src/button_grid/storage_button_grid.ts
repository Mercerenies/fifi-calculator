
import { AbstractButtonManager, ButtonGrid, GridCell } from "../button_grid.js";
import { SubcommandBehavior } from './subcommand.js';
import { ButtonModifiers } from './modifier_delegate.js';
import { backButton, Button } from './button.js';
import { variableNameInput } from '../input_box/algebraic_input.js';
import { TAURI } from '../tauri_api.js';

export class StorageButtonGrid extends ButtonGrid {
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
        new VariableStoreButton(":=", "t", false),
        new VariableStoreButton(":=<sup>K</sup>", "s", true),
      ],
      [
        new VariableUnbindButton(),
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
  private overrideKeepModifier: boolean;

  constructor(label: string | HTMLElement, key: string | null, overrideKeepModifier: boolean = false) {
    super(label, key);
    this.overrideKeepModifier = overrideKeepModifier;
  }

  async fire(manager: AbstractButtonManager): Promise<void> {
    // Fire-and-forget a new promise that gets user input, so we don't
    // hold up the existing input.
    this.readAndStore(manager);
  }

  asSubcommand(): SubcommandBehavior {
    return "invalid";
  }

  private async readAndStore(manager: AbstractButtonManager): Promise<void> {
    try {
      const isValid = await TAURI.validateStackSize(1);
      if (!isValid) {
        return;
      }
      const variableName = await variableNameInput(manager.inputManager);
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
  constructor() {
    super("<math><mo lspace='0' rspace='0'>↚</mo></math>", "u");
  }

  async fire(manager: AbstractButtonManager): Promise<void> {
    // Fire-and-forget a new promise that gets user input, so we don't
    // hold up the existing input.
    this.readAndStore(manager);
  }

  asSubcommand(): SubcommandBehavior {
    return "invalid";
  }

  private async readAndStore(manager: AbstractButtonManager): Promise<void> {
    try {
      const variableName = await variableNameInput(manager.inputManager);
      if (!variableName) {
        return;
      }
      await manager.invokeMathCommand('unbind_var', [variableName]);
    } finally {
      manager.resetState();
    }
  }
}
