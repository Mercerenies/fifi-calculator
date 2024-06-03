
import { InputMethod, InputBoxSession } from '../input_box.js';
import { KeyResponse, KeyEventInput } from '../keyboard.js';

// General-purpose input method that responds to ESCAPE and Enter and nothing else.
export class FreeformInputMethod implements InputMethod {
  readonly labelHtml: string;

  constructor(labelHtml: string) {
    this.labelHtml = labelHtml;
  }

  async onKeyDown(input: KeyEventInput, session: InputBoxSession): Promise<KeyResponse> {
    const key = input.toEmacsSyntax();
    if (key === "Escape") {
      // Abort the input.
      input.event.preventDefault();
      session.cancel();
    } else if (key === "Enter") {
      input.event.preventDefault();
      session.submit();
    }
    return KeyResponse.BLOCK;
  }
}
