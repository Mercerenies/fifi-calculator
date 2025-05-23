
import { InputMethod, InputBoxSession } from '../input_box.js';
import { KeyResponse, KeyEventInput } from '../keyboard.js';

// General-purpose input method that responds to ESCAPE and Enter and nothing else.
export class FreeformInputMethod implements InputMethod {
  readonly labelHtml: string;
  readonly inputType: "text" | "number";

  constructor(labelHtml: string, inputType: "text" | "number" = "text") {
    this.labelHtml = labelHtml;
    this.inputType = inputType;
  }

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
    }
    return KeyResponse.SOFT_BLOCK;
  }
}
