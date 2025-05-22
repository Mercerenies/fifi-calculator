
import { AbstractButtonManager, ButtonGrid, GridCell } from "../button_grid.js";
import { backButton, DispatchButton, Button } from './button.js';
import { SubcommandBehavior } from './subcommand.js';
import { InputBoxManager } from '../input_box.js';
import { TAURI, Validator } from '../tauri_api.js';
import { FreeformInputMethod } from '../input_box/freeform_input.js';

export class DatetimeButtonGrid extends ButtonGrid {
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
        new DispatchButton("<span class='mathy-text'>-0</span>", "days_since_zero", "D"),
        new DispatchButton("J", "julian_day", "J"),
        new DispatchButton("U", "unix_time", "U"),
      ],
      [
        new DispatchButton("⏲", "now", "N"),
        new ConvertTimezoneButton(),
      ],
      [
        new DispatchButton("M", "newmonth", "M"),
        new DispatchButton("Y", "newyear", "Y"),
        new DispatchButton("W", "newweek", "W"),
      ],
      [
        new DispatchButton("Δ<sub>M</sub>", "incmonth", "I"),
      ],
      [
      ],
      [
        backButton(this.rootGrid),
      ],
    ];
  }
}

export class ConvertTimezoneButton extends Button {
  constructor() {
    super("✈", "C");
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
      const destTimezone = await timezoneInput(manager.inputManager);
      if (!destTimezone) {
        return;
      }
      await manager.invokeMathCommand('tzconvert', [destTimezone]);
    } finally {
      manager.resetState();
    }
  }
}

// Freeform input that validates as a valid timezone.
export async function timezoneInput(manager: InputBoxManager, initialInput: string = ""): Promise<string | undefined> {
  const text = await manager.show(new FreeformInputMethod("New timezone:"), initialInput);
  if (!text) {
    return undefined;
  }
  const isValid = await TAURI.validateValue(text, Validator.IS_TIMEZONE);
  if (isValid) {
    return text;
  } else {
    return undefined;
  }
}
