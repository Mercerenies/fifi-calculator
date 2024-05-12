
import { KeyResponse, InputMethod, InputBoxManager } from '../input_box.js';

const tauri = window.__TAURI__.tauri;

// Input method that accepts numerical input.
export class NumericalInputMethod extends InputMethod {
  // TODO Get this from somewhere automated.
  static AUTO_SUBMIT_KEYS = new Set(["*", "/"]);

  getLabelHTML() { return "#:"; }

  private async submit(manager: InputBoxManager): Promise<void> {
    const text = manager.getTextBoxValue();
    if (text !== "") {
      await tauri.invoke('submit_number', { value: text });
    }
    manager.hide();
  }

  async onKeyDown(event: KeyboardEvent, manager: InputBoxManager): Promise<KeyResponse> {
    if (event.key === "Escape") {
      // Abort the input.
      event.preventDefault();
      manager.hide();
      return KeyResponse.BLOCK;
    } else if (event.key === "Enter") {
      event.preventDefault();
      await this.submit(manager);
      return KeyResponse.BLOCK;
    } else if (this.shouldAutoSubmit(event.key, manager)) {
      // Submit and perform a top-level command.
      await this.submit(manager);
      return KeyResponse.PASS;
    } else {
      // Absorb the input into the textbox.
      return KeyResponse.BLOCK;
    }
  }

  private shouldAutoSubmit(key: string, manager: InputBoxManager): boolean {
    if (NumericalInputMethod.AUTO_SUBMIT_KEYS.has(key)) {
      return true;
    }
    // + and - are special cases. They should *usually* auto-submit,
    // unless we're currently entering a number in scientific notation
    // (in which case, they're valid inputs in the text box).
    if ((key == '+') || (key == '-')) {
      return !manager.getTextBoxValue().endsWith("e");
    }
    return false;
  }
}
