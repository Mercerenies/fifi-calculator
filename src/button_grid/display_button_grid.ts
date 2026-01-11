
import { AbstractButtonManager, ButtonGrid, GridCell } from "../button_grid.js";
import { SubcommandBehavior } from './subcommand.js';
import { backButton, Button, DispatchButton } from './button.js';
import { InputBoxManager } from '../input_box.js';
import { FreeformInputMethod } from '../input_box/freeform_input.js';
import { svg } from '../util.js';
import { TAURI, Validator } from "../tauri_api.js";
import { HelpPage } from '../help_manager.js';
import * as HelpLibrary from "../help_library.js";
import { SetDisplayRadixArgs as SetDisplayRadixHelpArgs } from '../help_library.js';

function imageSvg(): HTMLElement {
  return svg('assets/image.svg', {alt: "graphics"});
}

export class DisplayButtonGrid extends ButtonGrid {
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
        new DispatchButton("N", "set_basic_language_mode", "N", HelpLibrary.setBasicLang),
        new DispatchButton("ℬ", "set_fancy_language_mode", "B", HelpLibrary.setFancyLang),
      ],
      [
        new SetDisplayRadixButton("0", "0", 10, "decimal"),
        new SetDisplayRadixButton("0x", "6", 16, "hexadecimald"),
        new SetDisplayRadixButton("0b", "2", 2, "binary"),
        new SetDisplayRadixButton("0o", "8", 8, "octal"),
        new SetDisplayRadixToInputButton(),
      ],
      [],
      [],
      [
        new DispatchButton(imageSvg(), "toggle_graphics", "G"),
        new DispatchButton("¶", "toggle_unicode", "u"),
      ],
      [
        backButton(this.rootGrid),
      ],
    ];
  }
}

// Button to set the display radix to a given constant.
export class SetDisplayRadixButton extends Button {
  readonly commandName: string = "set_display_radix";
  readonly targetRadix: number;
  readonly abbreviation: string | undefined;

  constructor(label: string | HTMLElement, keyboardShortcut: string | null, targetRadix: number, abbreviation?: string) {
    super(label, keyboardShortcut);
    if (!isValidRadix(targetRadix)) {
      throw new Error("Invalid radix for display radix button: " + targetRadix);
    }
    this.targetRadix = targetRadix;
    this.abbreviation = abbreviation;
  }

  async fire(manager: AbstractButtonManager): Promise<void> {
    await manager.invokeMathCommand(this.commandName, [String(this.targetRadix)]);
    manager.resetState();
  }

  asSubcommand(): SubcommandBehavior {
    return "invalid";
  }

  override getHelpPage(): HelpPage {
    return HelpLibrary.setDisplayRadixFunc({
      base: this.targetRadix,
      abbreviation: this.abbreviation,
    });
  }
}

// Button to set the display radix to a value given by user input.
export class SetDisplayRadixToInputButton extends Button {
  readonly commandName: string = "set_display_radix";

  constructor() {
    super("r", "r");
  }

  async fire(manager: AbstractButtonManager): Promise<void> {
    this.getInputAndSet(manager); // Fire-and-forget
  }

  asSubcommand(): SubcommandBehavior {
    return "invalid";
  }

  private async getInputAndSet(manager: AbstractButtonManager): Promise<void> {
    try {
      const userInput = await getRadixUserInput(manager.inputManager);
      if (userInput !== undefined) {
        await manager.invokeMathCommand(this.commandName, [userInput]);
      }
    } finally {
      manager.resetState();
    }
  }
}

function isValidRadix(n: number): boolean {
  return (n >= 2 && n <= 36 && Number.isInteger(n));
}

async function getRadixUserInput(manager: InputBoxManager): Promise<string | undefined> {
  const input = await manager.show(new FreeformInputMethod("Radix:", "number"));
  if (!input) {
    return undefined;
  }
  const isValid = await TAURI.validateValue(input, Validator.RADIX);
  if (isValid) {
    return input;
  } else {
    return undefined;
  }
}
