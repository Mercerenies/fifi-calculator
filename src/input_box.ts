
// Manager class for the input textbox.

import { KeyEventInput, KeyResponse } from './keyboard.js';

export class InputBoxManager {
  private inputMethod: InputMethod = NullaryInputMethod.INSTANCE;
  private inputSession: InputBoxSession;
  private inputBox: HTMLDivElement;
  private inputTextBox: HTMLInputElement;
  private inputLabel: HTMLElement;

  constructor(args: InputBoxManagerConstructorArgs) {
    this.inputBox = args.inputBox;
    this.inputTextBox = args.inputTextBox;
    this.inputLabel = args.inputLabel;

    // Initialize an empty session that does nothing on
    // resolve/reject.
    const ignore = function() {
      // Do nothing on resolve/reject.
    };
    this.inputSession = new ConcreteSession(this, ignore, ignore);
  }

  initListeners(): void {
    this.inputTextBox.addEventListener("focusout", () => this.onFocusOut());
  }

  isShowing(): boolean {
    return (this.inputBox.style.visibility !== 'hidden');
  }

  show(inputMethod: InputMethod, initialInput: string = ""): Promise<string | undefined> {
    return new Promise<string | undefined>((resolve, reject) => {
      this.inputMethod = inputMethod;
      this.inputSession = new ConcreteSession(this, resolve, reject);
      this.inputBox.style.visibility = 'visible';
      this.inputTextBox.type = inputMethod.inputType;
      this.inputLabel.innerHTML = inputMethod.labelHtml;
      this.setTextBoxValue(initialInput);
      window.setTimeout(() => this.inputTextBox.focus(), 1);
    }).then((s) => {
      this.hide();
      return s;
    });
  }

  hide(): void {
    this.inputBox.style.visibility = 'hidden';
    this.inputTextBox.value = "";
  }

  async onKeyDown(event: KeyEventInput): Promise<KeyResponse> {
    if (this.isShowing()) {
      return await this.inputMethod.onKeyDown(event, this.inputSession);
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
    if (!this.inputMethod.persistsWhenUnfocused) {
      this.hide();
      this.inputSession.cancel();
    }
  }
}

export interface InputBoxManagerConstructorArgs {
  inputBox: HTMLDivElement;
  inputTextBox: HTMLInputElement;
  inputLabel: HTMLElement;
}

export interface InputBoxSession {
  getText(): string;
  submit(): void;
  cancel(): void;
  reject(error: Error): void;
}

class ConcreteSession implements InputBoxSession {
  private inputBoxManager: InputBoxManager;
  private resolveFunc: (text: string | undefined) => void;
  private rejectFunc: (reason: Error) => void;

  constructor(
    inputBoxManager: InputBoxManager,
    resolveFunc: (text: string | undefined) => void,
    rejectFunc: (reason: Error) => void,
  ) {
    this.inputBoxManager = inputBoxManager;
    this.resolveFunc = resolveFunc;
    this.rejectFunc = rejectFunc;
  }

  getText(): string {
    return this.inputBoxManager.getTextBoxValue();
  }

  submit(): void {
    this.resolveFunc(this.getText());
  }

  cancel(): void {
    this.resolveFunc(undefined);
  }

  reject(error: Error): void {
    this.rejectFunc(error);
  }
}

export interface InputMethod {
  // If this is present and true, then the textbox will NOT hide when
  // the user clicks away from the textbox without submitting.
  readonly persistsWhenUnfocused?: boolean;
  readonly labelHtml: string;
  readonly inputType: "text" | "number";

  onKeyDown(event: KeyEventInput, session: InputBoxSession): Promise<KeyResponse>;
}

// Null Object implementation of InputMethod.
export class NullaryInputMethod implements InputMethod {
  labelHtml: string = "";
  inputType: "text" = "text" as const;

  async onKeyDown(): Promise<KeyResponse> {
    return KeyResponse.PASS;
  }

  static INSTANCE = new NullaryInputMethod();
};
