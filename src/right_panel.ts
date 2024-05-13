
// Right panel of the screen.

import { ButtonGridManager } from "./button_grid.js";
import { KeyEventInput } from "./keyboard.js";

// Currently this is just a trivial wrapper around ButtonGridManager.
// But we'll add more with prefix arguments.
export class RightPanelManager {
  private buttonGrid: ButtonGridManager;

  constructor(buttonGrid: ButtonGridManager) {
    this.buttonGrid = buttonGrid;
  }

  async onKeyDown(input: KeyEventInput): Promise<void> {
    await this.buttonGrid.onKeyDown(input);
  }
}
