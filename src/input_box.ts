
// Manager class for the input textbox.

const tauri = window.__TAURI__.tauri;

export class InputBoxManager {
  private inputMethod: InputMethod = NullaryInputMethod.INSTANCE;
  private inputBox: HTMLDivElement;
  private inputTextBox: HTMLInputElement;
  private inputLabel: HTMLElement;

  constructor(args: InputBoxManagerConstructorArgs) {
    this.inputBox = args.inputBox;
    this.inputTextBox = args.inputTextBox;
    this.inputLabel = args.inputLabel;
  }

  initListeners(): void {
    this.inputTextBox.addEventListener("focusout", () => this.onFocusOut());
  }

  isShowing(): boolean {
    return (this.inputBox.style.display !== 'none');
  }

  show(inputMethod: InputMethod): void {
    this.inputMethod = inputMethod;
    this.inputBox.style.display = 'flex';
    this.inputLabel.innerHTML = inputMethod.getLabelHTML();
    window.setTimeout(() => this.inputTextBox.focus(), 1);
  }

  hide(): void {
    this.inputBox.style.display = 'none';
    this.inputTextBox.value = "";
  }

  async onKeyDown(event: KeyboardEvent): Promise<KeyResponse> {
    if (this.isShowing()) {
      return await this.inputMethod.onKeyDown(event, this);
    } else {
      return KeyResponse.PASS;
    }
  }

  getTextBoxValue(): string {
    return this.inputTextBox.value;
  }

  setTextBoxValue(text: string): void {
    this.inputTextBox.value = text;
  }

  private onFocusOut(): void {
    if (this.inputMethod.shouldHideOnUnfocus()) {
      this.hide();
    }
  }
}

export interface InputBoxManagerConstructorArgs {
  inputBox: HTMLDivElement;
  inputTextBox: HTMLInputElement;
  inputLabel: HTMLElement;
}

export abstract class InputMethod {
  shouldHideOnUnfocus(): boolean {
    return true;
  }

  abstract onKeyDown(event: KeyboardEvent, manager: InputBoxManager): Promise<KeyResponse>;
  abstract getLabelHTML(): string;
}

// Null Object implementation of InputMethod.
export class NullaryInputMethod extends InputMethod {
  getLabelHTML() { return ""; }

  async onKeyDown(event: KeyboardEvent, manager: InputBoxManager): Promise<KeyResponse> {
    return KeyResponse.PASS;
  }

  static INSTANCE = new NullaryInputMethod();
};

// Input method that accepts numerical input. (TODO Currently just accepts integers)
export class NumericalInputMethod extends InputMethod {
  static VALID_INPUT_KEYS = new Set(["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"]);

  getLabelHTML() { return "#:"; }

  private async submit(manager: InputBoxManager): Promise<void> {
    const text = manager.getTextBoxValue();
    await tauri.invoke('submit_integer', { value: +text }); // TODO Support floats
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
    } else if (NumericalInputMethod.VALID_INPUT_KEYS.has(event.key)) {
      // Allow the input but block propagation.
      return KeyResponse.BLOCK;
    } else {
      // For unrecognized keys, submit and propagate.
      await this.submit(manager);
      return KeyResponse.PASS;
    }
  }
}

// Response to a keydown event.
export enum KeyResponse {
  // Pass the key input onto the parent container, outside of the
  // input box's control. Note that this does NOT imply that the input
  // box ignored the input, only that it wishes for the parent to see
  // it.
  PASS,
  // Suppress the input and do not allow parent containers to see it.
  BLOCK,
}
