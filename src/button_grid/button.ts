
import { ButtonGridManager, GridCell } from "../button_grid.js";
import { InputBoxManager } from '../input_box.js';
import { NumericalInputMethod } from '../input_box/numerical_input.js';

export abstract class Button implements GridCell {
  readonly label: string | HTMLElement;
  readonly keyboardShortcut: string | null;

  constructor(label: string | HTMLElement, keyboardShortcut: string | null) {
    this.label = label;
    this.keyboardShortcut = keyboardShortcut;
  }

  getHTML(manager: ButtonGridManager): HTMLElement {
    const button = document.createElement("button");
    if (typeof this.label === "string") {
      button.innerHTML = this.label;
    } else {
      button.appendChild(this.label);
    }
    button.addEventListener("click", () => this.fire(manager));
    return button;
  }

  abstract fire(manager: ButtonGridManager): Promise<void>;
}

export class DispatchButton extends Button {
  readonly commandName: string;

  constructor(label: string | HTMLElement, commandName: string, keyboardShortcut: string | null) {
    super(label, keyboardShortcut);
    this.commandName = commandName;
  }

  async fire(manager: ButtonGridManager): Promise<void> {
    manager.invokeMathCommand(this.commandName);
    manager.resetModifiers();
  }
}

export class NumericalInputButton extends Button {
  private inputManager: InputBoxManager;

  constructor(inputManager: InputBoxManager) {
    super("<span class='mathy-text'>#</span>", null);
    this.inputManager = inputManager;
  }

  async fire(manager: ButtonGridManager): Promise<void> {
    this.inputManager.show(new NumericalInputMethod(), "");
    manager.resetModifiers();
  }
}
