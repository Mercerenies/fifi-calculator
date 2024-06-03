
import { ButtonGridManager, ButtonGrid, GridCell } from "../button_grid.js";
import { svg } from '../util.js';

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
    manager.resetState();
  }
}

export class GotoButton extends Button {
  private gridFactory: () => ButtonGrid;

  constructor(label: string | HTMLElement, keyboardShortcut: string | null, gridFactory: ButtonGrid | (() => ButtonGrid)) {
    super(label, keyboardShortcut);
    if (typeof gridFactory === 'function') {
      this.gridFactory = gridFactory;
    } else {
      this.gridFactory = () => gridFactory;
    }
  }

  async fire(manager: ButtonGridManager): Promise<void> {
    const grid = this.gridFactory();
    console.log(grid);
    manager.setActiveGrid(grid);
  }
}

export function backButton(gridFactory: ButtonGrid | (() => ButtonGrid)): GotoButton {
  const image = svg('assets/back.svg', {alt: 'back'});
  return new GotoButton(image, "ESC", gridFactory);
}
