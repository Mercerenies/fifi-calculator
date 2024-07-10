
import { ButtonGridManager, ButtonGrid, GridCell } from "../button_grid.js";
import { backButton, Button, DispatchButton } from './button.js';
import { InputBoxManager } from '../input_box.js';
import { FreeformInputMethod } from '../input_box/freeform_input.js';
import { svg } from '../util.js';
import { TAURI, Validator, StackQueryType } from '../tauri_api.js';

function rulerSvg(): HTMLElement {
  return svg('assets/ruler.svg', {alt: "convert"});
}

export class UnitsButtonGrid extends ButtonGrid {
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
        new UnitConversionButton(this.inputManager),
      ],
      [
        new DispatchButton("1", "remove_units", "r"),
        new DispatchButton("cm", "extract_units", "x"),
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

export class UnitConversionButton extends Button {
  private inputManager: InputBoxManager;

  constructor(inputManager: InputBoxManager) {
    super(rulerSvg(), "c");
    this.inputManager = inputManager;
  }

  async fire(manager: ButtonGridManager): Promise<void> {
    // Fire-and-forget a new promise that gets user input, so we don't
    // hold up the existing input.
    this.readAndSubstitute(manager);
  }

  private async readAndSubstitute(manager: ButtonGridManager): Promise<void> {
    try {
      const isValid = await TAURI.validateStackSize(1);
      if (!isValid) {
        return;
      }

      if (await TAURI.queryStack({ stackIndex: 0, queryType: StackQueryType.HAS_UNITS })) {
        const destUnits = await unitInput(this.inputManager, "New units:");
        if (!destUnits) {
          return;
        }
        await manager.invokeMathCommand('convert_units_with_context', [destUnits]);
      } else {
        const sourceUnits = await unitInput(this.inputManager, "Old units:");
        if (!sourceUnits) {
          return;
        }
        const destUnits = await unitInput(this.inputManager, "New units:");
        if (!destUnits) {
          return;
        }
        await manager.invokeMathCommand('convert_units', [sourceUnits, destUnits]);
      }
    } finally {
      manager.resetState();
    }
  }
}

// Freeform input that validates as a valid unit.
export async function unitInput(manager: InputBoxManager, prompt: string, initialInput: string = ""): Promise<string | undefined> {
  const text = await manager.show(new FreeformInputMethod(prompt), initialInput);
  if (!text) {
    return undefined;
  }
  const isValid = await TAURI.validateValue(text, Validator.ALL_UNITS);
  if (isValid) {
    return text;
  } else {
    return undefined;
  }
}
