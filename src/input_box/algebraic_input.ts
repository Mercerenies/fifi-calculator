
import { InputMethod, InputBoxManager } from '../input_box.js';
import { KeyResponse, KeyEventInput } from '../keyboard.js';

const tauri = window.__TAURI__.tauri;

// Input method that accepts expression-based input.
export class AlgebraicInputMethod extends InputMethod {

  getLabelHTML() { return "Alg:"; }

  private async submit(manager: InputBoxManager): Promise<void> {
    const text = manager.getTextBoxValue();
    if (text !== "") {
      await tauri.invoke('submit_expr', { value: text });
    }
    manager.hide();
  }

  async onKeyDown(input: KeyEventInput, manager: InputBoxManager): Promise<KeyResponse> {
    const key = input.toEmacsSyntax();
    if (key === "Escape") {
      // Abort the input.
      input.event.preventDefault();
      manager.hide();
    } else if (key === "Enter") {
      input.event.preventDefault();
      await this.submit(manager);
    }
    return KeyResponse.BLOCK;
  }
}
