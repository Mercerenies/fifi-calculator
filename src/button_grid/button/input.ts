
import { ButtonGridManager } from "../../button_grid.js";
import { modifiersToRustArgs } from "../modifier_delegate.js";
import { Button } from '../button.js';
import { InputBoxManager } from '../../input_box.js';
import { numericalInputToStack } from '../../input_box/numerical_input.js';
import { algebraicInputToStack, editStackFrame } from '../../input_box/algebraic_input.js';
import { stringInputToStack } from '../../input_box/string_input.js';
import { svg } from '../../util.js';

function pencilSvg(): HTMLElement {
  return svg('assets/pencil.svg', {alt: 'edit'});
}

export abstract class InputButton extends Button {
  private inputManager: InputBoxManager;

  constructor(label: string | HTMLElement, keyboardShortcut: string | null, inputManager: InputBoxManager) {
    super(label, keyboardShortcut);
    this.inputManager = inputManager;
  }

  abstract runInputFlow(inputManager: InputBoxManager, gridManager: ButtonGridManager): Promise<void>;

  async fire(manager: ButtonGridManager): Promise<void> {
    await this.runInputFlow(this.inputManager, manager);
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

// The flow for this button is sort of unique, since we have to get
// stack data (using the prefix arg), then get user input, and finally
// run a command. So the handling of the modifiers is done in the
// frontend class for this particular command. Most commands do this
// in the backend.
export class AlgebraicEditButton extends InputButton {
  constructor(inputManager: InputBoxManager) {
    super(pencilSvg(), "`", inputManager);
  }

  runInputFlow(manager: InputBoxManager, gridManager: ButtonGridManager): Promise<void> {
    const modifiers = gridManager.getModifiers();
    const stackIndex = Math.max(modifiers.prefixArgument ?? 1, 1) - 1;
    return editStackFrame(manager, stackIndex, {
      isMouseInteraction: false,
      commandOptionsOverride: modifiersToRustArgs(modifiers),
    });
  }
}
