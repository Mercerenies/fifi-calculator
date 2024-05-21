
// Right panel of the screen.

import { ButtonGridManager, ButtonGrid } from "./button_grid.js";
import { KeyEventInput } from "./keyboard.js";
import { PrefixArgStateMachine } from "./prefix_argument.js";
import { PrefixArgumentDelegate } from "./prefix_argument/prefix_delegate.js";
import { PrefixArgumentDisplay } from "./prefix_argument/display.js";
import { UndoManager } from './undo_manager.js';
import { KeyResponse } from './keyboard.js';

export class RightPanelManager {
  readonly prefixArgStateMachine: PrefixArgStateMachine;
  readonly buttonGrid: ButtonGridManager;
  readonly prefixArgDisplay: PrefixArgumentDisplay;
  readonly undoManager: UndoManager;

  constructor(args: RightPanelArguments) {
    this.prefixArgStateMachine = new PrefixArgStateMachine();
    this.buttonGrid = new ButtonGridManager(
      args.buttonGrid,
      args.initialGrid,
      new PrefixArgumentDelegate(this.prefixArgStateMachine),
    );
    this.prefixArgDisplay = new PrefixArgumentDisplay(
      args.prefixPanel,
      this.prefixArgStateMachine,
      args,
    );
    this.undoManager = new UndoManager(args);
  }

  async onKeyDown(input: KeyEventInput): Promise<void> {
    const blocked = await this.undoManager.onKeyDown(input);
    if (blocked !== KeyResponse.BLOCK) {
      await this.buttonGrid.onKeyDown(input);
    }
  }

  initListeners() {
    this.buttonGrid.initListeners();
    this.prefixArgDisplay.initListeners();
    this.undoManager.initListeners();
  }
}

export interface RightPanelArguments {
  buttonGrid: HTMLElement,
  prefixPanel: HTMLElement,
  initialGrid: ButtonGrid,
  undoButton: HTMLButtonElement,
  redoButton: HTMLButtonElement,
  displayBoxId?: string,
}
