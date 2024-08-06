
import { AbstractButtonManager, ButtonGrid, GridCell } from "../button_grid.js";
import { subcommandStr } from '../tauri_api.js';
import { modifiersToRustArgs } from './modifier_delegate.js';
import { SubcommandBehavior, SubcommandButtonManager,
         SubcommandButtonManagerOpts, IsSubcommand } from './subcommand.js';
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
  private opts: Partial<SubcommandDispatchButtonOpts>;

  readonly commandName: string;

  constructor(
    label: string | HTMLElement,
    commandName: string,
    keyboardShortcut: string | null,
    opts: Partial<SubcommandDispatchButtonOpts> = {},
  ) {
    super(label, keyboardShortcut);
    this.commandName = commandName;
    this.opts = opts;
  }

  async fire(manager: AbstractButtonManager): Promise<void> {
    const mainCommandModifiers = manager.getModifiers();
    SubcommandButtonManager.queryForSubcommand(manager, async (subcommandId) => {
      await manager.invokeMathCommand(this.commandName, [subcommandStr(subcommandId)], mainCommandModifiers);
    }, this.opts);
  }

  asSubcommand(): SubcommandBehavior {
    return "invalid";
  }
}

export class DoubleSubcommandDispatchButton extends Button {
  private opts: Partial<DoubleSubcommandDispatchButtonOpts>;

  readonly commandName: string;

  constructor(
    label: string | HTMLElement,
    commandName: string,
    keyboardShortcut: string | null,
    opts: Partial<DoubleSubcommandDispatchButtonOpts> = {},
  ) {
    super(label, keyboardShortcut);
    this.commandName = commandName;
    this.opts = opts;
  }

  async fire(manager: AbstractButtonManager): Promise<void> {
    const mainCommandModifiers = manager.getModifiers();
    SubcommandButtonManager.queryForSubcommand(manager, async (subcommandId1) => {
      SubcommandButtonManager.queryForSubcommand(manager, async (subcommandId2) => {
        await manager.invokeMathCommand(
          this.commandName,
          [subcommandStr(subcommandId1), subcommandStr(subcommandId2)],
          mainCommandModifiers,
        );
      }, this.subcommandOpts2());
    }, this.subcommandOpts1());
  }

  private subcommandOpts1(): Partial<SubcommandButtonManagerOpts> {
    if (this.opts.firstLabelHTML) {
      return { labelHTML: this.opts.firstLabelHTML };
    } else {
      return {};
    }
  }

  private subcommandOpts2(): Partial<SubcommandButtonManagerOpts> {
    if (this.opts.secondLabelHTML) {
      return { labelHTML: this.opts.secondLabelHTML };
    } else {
      return {};
    }
  }

  asSubcommand(): SubcommandBehavior {
    return "invalid";
  }
}

export interface SubcommandDispatchButtonOpts {
  labelHTML: string;
}


export interface DoubleSubcommandDispatchButtonOpts {
  firstLabelHTML: string;
  secondLabelHTML: string;
}
