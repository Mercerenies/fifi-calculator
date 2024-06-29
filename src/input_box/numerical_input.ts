
import { InputMethod, InputBoxSession, InputBoxManager } from '../input_box.js';
import { KeyResponse, KeyEventInput } from '../keyboard.js';
import { TAURI, defaultCommandOptions } from '../tauri_api.js';

// Input method that accepts numerical input.
export class NumericalInputMethod implements InputMethod {
  // TODO Get this from somewhere automated.
  static AUTO_SUBMIT_KEYS = new Set(["*", "/", "^"]);

  labelHtml: string = "#:";
  inputType: "number" = "number" as const;

  async onKeyDown(input: KeyEventInput, session: InputBoxSession): Promise<KeyResponse> {
    const key = input.toEmacsSyntax();
    if (key === "Escape") {
      // Abort the input.
      input.event.preventDefault();
      session.cancel();
      return KeyResponse.BLOCK;
    } else if (key === "Enter") {
      input.event.preventDefault();
      session.submit();
      return KeyResponse.BLOCK;
    } else if (this.shouldAutoSubmit(key, session)) {
      // Submit and perform a top-level command.
      session.submit();
      return KeyResponse.PASS;
    } else {
      // Absorb the input into the textbox.
      return KeyResponse.BLOCK;
    }
  }

  private shouldAutoSubmit(key: string, session: InputBoxSession): boolean {
    if (NumericalInputMethod.AUTO_SUBMIT_KEYS.has(key)) {
      return true;
    }
    // + and - are special cases. They should *usually* auto-submit,
    // unless we're currently entering a number in scientific notation
    // (in which case, they're valid inputs in the text box).
    if ((key == '+') || (key == '-')) {
      return !session.getText().endsWith("e");
    }
    return false;
  }
}

export async function numericalInputToStack(manager: InputBoxManager, initialInput: string = ""): Promise<void> {
  const text = await manager.show(new NumericalInputMethod(), initialInput);
  if (text) {
    await TAURI.runMathCommand('push_number', [text], defaultCommandOptions());
  }
}
