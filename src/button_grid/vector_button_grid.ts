
import { ButtonGridManager, ButtonGrid, GridCell } from "../button_grid.js";
import { Button, backButton, DispatchButton } from './button.js';
import { InputBoxManager } from '../input_box.js';
import { FreeformInputMethod } from '../input_box/freeform_input.js';
import { TAURI, Validator } from '../tauri_api.js';

export class VectorButtonGrid extends ButtonGrid {
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
        new DispatchButton("p", "pack", "p"),
        new DispatchButton("u", "unpack", "u"),
        new DispatchButton("<math><mi>ι</mi></math>", "iota", "x"),
        new DispatchButton("<math><mo>*</mo></math>", "repeat", "b"),
      ],
      [
        new DispatchButton("++", "vconcat", "|"),
        new DispatchButton("::", "cons", "k"),
        new DispatchButton("1<sup>st</sup>", "head", "h"),
        new GetNthButton(this.inputManager),
      ],
      [
        new DispatchButton("↘", "diag", "d"),
        new IdentityMatrixButton(this.inputManager),
      ],
      [
        new DispatchButton("[", "incomplete[", "["),
        new DispatchButton("]", "incomplete]", "]"),
      ],
      [
        new DispatchButton("(", "incomplete(", "("),
        new DispatchButton(")", "incomplete)", ")"),
      ],
      [
        backButton(this.rootGrid),
      ],
    ];
  }
}

// If given a numerical argument, uses that as the sole argument.
// Otherwise, prompts for a nonnegative integer.
export class IdentityMatrixButton extends Button {
  readonly commandName: string = "identity_matrix";
  private inputManager: InputBoxManager;

  constructor(inputManager: InputBoxManager) {
    super("<math><msub><mi>I</mi><mi>n</mi></msub></math>", "i");
    this.inputManager = inputManager;
  }

  async fire(manager: ButtonGridManager): Promise<void> {
    // Fire-and-forget a new promise that gets user input, so we don't
    // hold up the existing input.
    this.readAndRun(manager);
  }

  private async readAndRun(manager: ButtonGridManager): Promise<void> {
    try {
      let numericalArg = manager.getModifiers().prefixArgument ?? null;
      if (numericalArg === null) {
        numericalArg = await readUsize(this.inputManager);
        if (numericalArg === null) {
          return;
        }
      }

      // Ensure the numerical argument (if provided) is nonnegative.
      numericalArg = Math.abs(numericalArg);

      await manager.invokeMathCommand(this.commandName, [String(numericalArg)]);
    } finally {
      manager.resetState();
    }
  }
}

// If given a numerical argument, uses that as the sole argument.
// Otherwise, prompts for a nonnegative integer.
export class GetNthButton extends Button {
  readonly commandName: string = "nth";
  private inputManager: InputBoxManager;

  constructor(inputManager: InputBoxManager) {
    super("n<sup>th</sup>", "r");
    this.inputManager = inputManager;
  }

  async fire(manager: ButtonGridManager): Promise<void> {
    // Fire-and-forget a new promise that gets user input, so we don't
    // hold up the existing input.
    this.readAndRun(manager);
  }

  private async readAndRun(manager: ButtonGridManager): Promise<void> {
    try {
      let numericalArg = manager.getModifiers().prefixArgument ?? null;
      if (numericalArg === null) {
        numericalArg = await readI64(this.inputManager);
        if (numericalArg === null) {
          return;
        }
      }

      await manager.invokeMathCommand(this.commandName, [String(numericalArg)]);
    } finally {
      manager.resetState();
    }
  }
}

async function readUsize(inputManager: InputBoxManager): Promise<number | null> {
  const value = await inputManager.show(new FreeformInputMethod("Dims:", "number"));
  if (!value) {
    return null;
  }
  if (!await TAURI.validateValue(value, Validator.USIZE)) {
    return null;
  }
  return Number(value);
}

async function readI64(inputManager: InputBoxManager): Promise<number | null> {
  const value = await inputManager.show(new FreeformInputMethod("Index:", "number"));
  if (!value) {
    return null;
  }
  if (!await TAURI.validateValue(value, Validator.I64)) {
    return null;
  }
  return Number(value);
}
