
import { ButtonGridManager } from "../../button_grid.js";
import { Button } from '../button.js';
import { InputBoxManager } from '../../input_box.js';
import { numericalInputToStack } from '../../input_box/numerical_input.js';
import { algebraicInputToStack } from '../../input_box/algebraic_input.js';
import { stringInputToStack } from '../../input_box/string_input.js';

export abstract class InputButton extends Button {
  private inputManager: InputBoxManager;

  constructor(label: string | HTMLElement, keyboardShortcut: string | null, inputManager: InputBoxManager) {
    super(label, keyboardShortcut);
    this.inputManager = inputManager;
  }

  abstract runInputFlow(manager: InputBoxManager): Promise<void>;

  async fire(manager: ButtonGridManager): Promise<void> {
    await this.runInputFlow(this.inputManager);
    manager.resetState();
  }
}

export class NumericalInputButton extends InputButton {
  constructor(inputManager: InputBoxManager) {
    super("<span class='mathy-text'>#</span>", null, inputManager);
  }

  runInputFlow(manager: InputBoxManager): Promise<void> {
    return numericalInputToStack(manager, "");
  }
}

export class AlgebraicInputButton extends InputButton {
  constructor(inputManager: InputBoxManager) {
    super("<math><mi>f</mi></math>", "'", inputManager);
  }

  runInputFlow(inputManager: InputBoxManager): Promise<void> {
    return algebraicInputToStack(inputManager, "");
  }
}

export class StringInputButton extends InputButton {
  constructor(inputManager: InputBoxManager) {
    super("&quot;", '"', inputManager);
  }

  runInputFlow(manager: InputBoxManager): Promise<void> {
    return stringInputToStack(manager, "");
  }
}
