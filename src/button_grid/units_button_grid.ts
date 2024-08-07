
import { AbstractButtonManager, ButtonGrid, GridCell } from "../button_grid.js";
import { SubcommandBehavior } from './subcommand.js';
import { backButton, Button, DispatchButton } from './button.js';
import { InputBoxManager } from '../input_box.js';
import { FreeformInputMethod } from '../input_box/freeform_input.js';
import { svg } from '../util.js';
import { TAURI, Validator, StackQueryType } from '../tauri_api.js';

function rulerSvg(): HTMLElement {
  return svg('assets/ruler.svg', {alt: "convert"});
}

function thermometerSvg(): HTMLElement {
  return svg('assets/thermometer.svg', {alt: "convert temperature"});
}

export class UnitsButtonGrid extends ButtonGrid {
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
        new UnitConversionButton(),
        new TemperatureConversionButton(),
        new DispatchButton("<span class='mathy-text'>m=</span>", "simplify_units", "s"),
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
  constructor() {
    super(rulerSvg(), "c");
  }

  async fire(manager: AbstractButtonManager): Promise<void> {
    // Fire-and-forget a new promise that gets user input, so we don't
    // hold up the existing input.
    this.readAndSubstitute(manager);
  }

  asSubcommand(): SubcommandBehavior {
    return "invalid";
  }

  private async readAndSubstitute(manager: AbstractButtonManager): Promise<void> {
    try {
      const isValid = await TAURI.validateStackSize(1);
      if (!isValid) {
        return;
      }

      if (await TAURI.queryStack({ stackIndex: 0, queryType: StackQueryType.HAS_UNITS })) {
        const destUnits = await unitInput(manager.inputManager, "New units:");
        if (!destUnits) {
          return;
        }
        await manager.invokeMathCommand('convert_units_with_context', [destUnits]);
      } else {
        const sourceUnits = await unitInput(manager.inputManager, "Old units:");
        if (!sourceUnits) {
          return;
        }
        const destUnits = await unitInput(manager.inputManager, "New units:");
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

export class TemperatureConversionButton extends Button {
  constructor() {
    super(thermometerSvg(), "t");
  }

  async fire(manager: AbstractButtonManager): Promise<void> {
    // Fire-and-forget a new promise that gets user input, so we don't
    // hold up the existing input.
    this.readAndSubstitute(manager);
  }

  asSubcommand(): SubcommandBehavior {
    return "invalid";
  }

  private async readAndSubstitute(manager: AbstractButtonManager): Promise<void> {
    try {
      const isValid = await TAURI.validateStackSize(1);
      if (!isValid) {
        return;
      }

      if (await TAURI.queryStack({ stackIndex: 0, queryType: StackQueryType.HAS_BASIC_TEMPERATURE_UNITS })) {
        const destUnits = await temperatureUnitInput(manager.inputManager, "New temperature:");
        if (!destUnits) {
          return;
        }
        await manager.invokeMathCommand('convert_temp_with_context', [destUnits]);
      } else if (await TAURI.queryStack({ stackIndex: 0, queryType: StackQueryType.HAS_UNITS })) {
        // Units are present, but it's not a temperature expression.
        // This is an error.
        await TAURI.showError("Expected basic temperature expression");
        return;
      } else {
        const sourceUnits = await temperatureUnitInput(manager.inputManager, "Old temperature:");
        if (!sourceUnits) {
          return;
        }
        const destUnits = await temperatureUnitInput(manager.inputManager, "New temperature:");
        if (!destUnits) {
          return;
        }
        await manager.invokeMathCommand('convert_temp', [sourceUnits, destUnits]);
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

// Freeform input that validates as a valid temperature unit.
export async function temperatureUnitInput(manager: InputBoxManager, prompt: string, initialInput: string = ""): Promise<string | undefined> {
  const text = await manager.show(new FreeformInputMethod(prompt), initialInput);
  if (!text) {
    return undefined;
  }
  const isValid = await TAURI.validateValue(text, Validator.IS_TEMPERATURE_UNIT);
  if (isValid) {
    return text;
  } else {
    return undefined;
  }
}
