
import { ButtonGridManager } from "../../button_grid.js";
import { Button } from '../button.js';
import { InputBoxManager, InputMethod } from '../../input_box.js';
import { NumericalInputMethod } from '../../input_box/numerical_input.js';
import { AlgebraicInputMethod } from '../../input_box/algebraic_input.js';

export abstract class InputButton extends Button {
  private inputManager: InputBoxManager;

  constructor(label: string | HTMLElement, keyboardShortcut: string | null, inputManager: InputBoxManager) {
    super(label, keyboardShortcut);
    this.inputManager = inputManager;
  }

  abstract getInputMethod(): InputMethod;

  async fire(manager: ButtonGridManager): Promise<void> {
    this.inputManager.show(this.getInputMethod(), "");
    manager.resetModifiers();
  }
}

export class NumericalInputButton extends InputButton {
  constructor(inputManager: InputBoxManager) {
    super("<span class='mathy-text'>#</span>", null, inputManager);
  }

  getInputMethod(): NumericalInputMethod {
    return new NumericalInputMethod();
  }
}

export class AlgebraicInputButton extends InputButton {
  constructor(inputManager: InputBoxManager) {
    super("<math><mi>f</mi></math>", "'", inputManager);
  }

  getInputMethod(): AlgebraicInputMethod {
    return new AlgebraicInputMethod();
  }
}
