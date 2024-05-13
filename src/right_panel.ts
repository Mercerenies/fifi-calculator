
// Right panel of the screen.

import { ButtonGridManager, ButtonGrid } from "./button_grid.js";
import { KeyEventInput } from "./keyboard.js";
import { PrefixArgStateMachine } from "./prefix_argument.js";
import { PrefixArgumentDelegate } from "./prefix_argument/prefix_delegate.js";

// Currently this is just a trivial wrapper around ButtonGridManager.
// But we'll add more with prefix arguments.
export class RightPanelManager {
  private prefixArgStateMachine: PrefixArgStateMachine;
  private buttonGrid: ButtonGridManager;

  constructor(buttonGridElement: HTMLElement, initialGrid: ButtonGrid) {
    this.prefixArgStateMachine = new PrefixArgStateMachine();
    this.buttonGrid = new ButtonGridManager(
      buttonGridElement,
      initialGrid,
      new PrefixArgumentDelegate(this.prefixArgStateMachine),
    );
  }

  async onKeyDown(input: KeyEventInput): Promise<void> {
    await this.buttonGrid.onKeyDown(input);
  }
}
