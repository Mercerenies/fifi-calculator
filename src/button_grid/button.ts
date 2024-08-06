
import { AbstractButtonManager, ButtonGrid, GridCell } from "../button_grid.js";
import { subcommandStr } from '../tauri_api.js';
import { modifiersToRustArgs } from './modifier_delegate.js';
import { SubcommandBehavior, SubcommandButtonManager, IsSubcommand } from './subcommand.js';
import { svg } from '../util.js';

export abstract class Button implements GridCell {
  readonly label: string | HTMLElement;
  readonly keyboardShortcut: string | null;

  constructor(label: string | HTMLElement, keyboardShortcut: string | null) {
    this.label = label;
    this.keyboardShortcut = keyboardShortcut;
  }

  getHTML(manager: AbstractButtonManager): HTMLElement {
    const button = document.createElement("button");
    if (typeof this.label === "string") {
      button.innerHTML = this.label;
    } else {
      button.appendChild(this.label);
    }
    button.addEventListener("click", () => manager.onClick(this));
    return button;
  }

  abstract fire(manager: AbstractButtonManager): Promise<void>;

  abstract asSubcommand(manager: AbstractButtonManager): SubcommandBehavior;
}

export class DispatchButton extends Button {
  readonly commandName: string;

  constructor(label: string | HTMLElement, commandName: string, keyboardShortcut: string | null) {
    super(label, keyboardShortcut);
    this.commandName = commandName;
  }

  async fire(manager: AbstractButtonManager): Promise<void> {
    manager.invokeMathCommand(this.commandName);
    manager.resetState();
  }

  asSubcommand(manager: AbstractButtonManager): SubcommandBehavior {
    return new IsSubcommand({
      name: this.commandName,
      options: modifiersToRustArgs(manager.getModifiers()),
    });
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

  async fire(manager: AbstractButtonManager): Promise<void> {
    const grid = this.gridFactory();
    manager.setActiveGrid(grid);
  }

  asSubcommand(): SubcommandBehavior {
    return "pass";
  }
}

export function backButton(gridFactory: ButtonGrid | (() => ButtonGrid)): GotoButton {
  const image = svg('assets/back.svg', {alt: 'back'});
  return new GotoButton(image, "Escape", gridFactory);
}

export class SubcommandDispatchButton extends Button {
  readonly commandName: string;

  constructor(label: string | HTMLElement, commandName: string, keyboardShortcut: string | null) {
    super(label, keyboardShortcut);
    this.commandName = commandName;
  }

  async fire(manager: AbstractButtonManager): Promise<void> {
    const mainCommandModifiers = manager.getModifiers();
    const subcommandManager = new SubcommandButtonManager(manager, async (subcommandId) => {
      await manager.invokeMathCommand(this.commandName, [subcommandStr(subcommandId)], mainCommandModifiers);
    })
    manager.setCurrentManager(subcommandManager);
    manager.resetState();
  }

  asSubcommand(): SubcommandBehavior {
    return "invalid";
  }
}
