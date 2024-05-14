
// Right panel of the screen.

import { ButtonGridManager, ButtonGrid } from "./button_grid.js";
import { KeyEventInput } from "./keyboard.js";
import { PrefixArgStateMachine } from "./prefix_argument.js";
import { PrefixArgumentDelegate } from "./prefix_argument/prefix_delegate.js";
import { PrefixArgumentDisplay } from "./prefix_argument/display.js";

// Currently this is just a trivial wrapper around ButtonGridManager.
// But we'll add more with prefix arguments.
export class RightPanelManager {
  private prefixArgStateMachine: PrefixArgStateMachine;
  private buttonGrid: ButtonGridManager;
  private prefixArgDisplay: PrefixArgumentDisplay;

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
  }

  async onKeyDown(input: KeyEventInput): Promise<void> {
    await this.buttonGrid.onKeyDown(input);
  }

  initListeners() {
    this.buttonGrid.initListeners();
    this.prefixArgDisplay.initListeners();
  }
}

export interface RightPanelArguments {
  buttonGrid: HTMLElement,
  prefixPanel: HTMLElement,
  initialGrid: ButtonGrid,
  displayBoxId?: string,
}
