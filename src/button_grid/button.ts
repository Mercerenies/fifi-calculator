
import { ButtonGridManager, GridCell } from "../button_grid.js";

const tauri = window.__TAURI__.tauri;

export abstract class Button implements GridCell {
  readonly label: string;
  readonly keyboardShortcut: string | null;

  constructor(label: string, keyboardShortcut: string | null) {
    this.label = label;
    this.keyboardShortcut = keyboardShortcut;
  }

  getHTML(manager: ButtonGridManager): HTMLElement {
    const button = document.createElement("button");
    button.innerHTML = this.label;
    button.addEventListener("click", () => this.fire(manager));
    return button;
  }

  abstract fire(manager: ButtonGridManager): Promise<void>;
}

export class DispatchButton extends Button {
  readonly commandName: string;

  constructor(label: string, commandName: string, keyboardShortcut: string | null) {
    super(label, keyboardShortcut);
    this.commandName = commandName;
  }

  fire(): Promise<void> {
    return tauri.invoke('math_command', { commandName: this.commandName });
  }
}
