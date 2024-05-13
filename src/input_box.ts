
// Manager class for the input textbox.

import { KeyEventInput, KeyResponse } from './keyboard.js';

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

  show(inputMethod: InputMethod, initialInput: string = ""): void {
    this.inputMethod = inputMethod;
    this.inputBox.style.display = 'flex';
    this.inputLabel.innerHTML = inputMethod.getLabelHTML();
    this.setTextBoxValue(initialInput);
    window.setTimeout(() => this.inputTextBox.focus(), 1);
  }

  hide(): void {
    this.inputBox.style.display = 'none';
    this.inputTextBox.value = "";
  }

  async onKeyDown(event: KeyEventInput): Promise<KeyResponse> {
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

  abstract onKeyDown(event: KeyEventInput, manager: InputBoxManager): Promise<KeyResponse>;
  abstract getLabelHTML(): string;
}

// Null Object implementation of InputMethod.
export class NullaryInputMethod extends InputMethod {
  getLabelHTML() { return ""; }

  async onKeyDown(): Promise<KeyResponse> {
    return KeyResponse.PASS;
  }

  static INSTANCE = new NullaryInputMethod();
};
