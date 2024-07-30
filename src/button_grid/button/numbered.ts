
import { ButtonGridManager } from "../../button_grid.js";
import { InputBoxManager } from '../../input_box.js';
import { FreeformInputMethod } from '../../input_box/freeform_input.js';
import { Button } from '../button.js';
import { TAURI, Validator } from '../../tauri_api.js';

// A NumberedButton is a button which uses the numerical argument (if
// provided), or prompts the user for one if not provided. Invokes the
// command with the given name.
export abstract class NumberedButton extends Button {
  readonly commandName: string;

  constructor(label: string | HTMLElement, commandName: string, keyboardShortcut: string | null) {
    super(label, keyboardShortcut);
    this.commandName = commandName;
  }

  async fire(manager: ButtonGridManager) {
    try {
      let numericalArg = manager.getModifiers().prefixArgument ?? null;
      if (numericalArg === null) {
        numericalArg = await this.getNumericalInput(manager.inputManager);
        if (numericalArg === null) {
          return;
        }
      }

      await manager.invokeMathCommand(this.commandName, [String(numericalArg)]);
    } finally {
      manager.resetState();
    }
  }

  abstract normalizeNumber(n: number): number;

  abstract getNumericalInput(inputManager: InputBoxManager): Promise<number | null>;
}

export class UnsignedNumberedButton extends NumberedButton {
  private prompt: string;

  constructor(label: string | HTMLElement, commandName: string, keyboardShortcut: string | null, prompt: string) {
    super(label, commandName, keyboardShortcut);
    this.prompt = prompt;
  }

  normalizeNumber(n: number) {
    return Math.abs(n);
  }

  async getNumericalInput(inputManager: InputBoxManager) {
    const value = await inputManager.show(new FreeformInputMethod(this.prompt, "number"));
    if (!value) {
      return null;
    }
    if (!await TAURI.validateValue(value, Validator.USIZE)) {
      return null;
    }
    return Number(value);
  }
}


export class SignedNumberedButton extends NumberedButton {
  private prompt: string;

  constructor(label: string | HTMLElement, commandName: string, keyboardShortcut: string | null, prompt: string) {
    super(label, commandName, keyboardShortcut);
    this.prompt = prompt;
  }

  normalizeNumber(n: number) {
    return n;
  }

  async getNumericalInput(inputManager: InputBoxManager) {
    const value = await inputManager.show(new FreeformInputMethod(this.prompt, "number"));
    if (!value) {
      return null;
    }
    if (!await TAURI.validateValue(value, Validator.I64)) {
      return null;
    }
    return Number(value);
  }
}
