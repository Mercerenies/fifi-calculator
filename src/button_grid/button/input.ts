
import { AbstractButtonManager } from "../../button_grid.js";
import { SubcommandBehavior } from '../subcommand.js';
import { modifiersToRustArgs } from "../modifier_delegate.js";
import { Button } from '../button.js';
import { numericalInputToStack } from '../../input_box/numerical_input.js';
import { algebraicInputToStack, editStackFrame } from '../../input_box/algebraic_input.js';
import { stringInputToStack } from '../../input_box/string_input.js';
import { svg } from '../../util.js';

function pencilSvg(): HTMLElement {
  return svg('assets/pencil.svg', {alt: 'edit'});
}

export abstract class InputButton extends Button {
  abstract runInputFlow(gridManager: AbstractButtonManager): Promise<void>;

  async fire(manager: AbstractButtonManager): Promise<void> {
    await this.runInputFlow(manager);
    manager.resetState();
  }

  /* eslint-disable-next-line @typescript-eslint/no-unused-vars */
  asSubcommand(manager: AbstractButtonManager): SubcommandBehavior {
    return "invalid";
  }
}

export class NumericalInputButton extends InputButton {
  constructor() {
    super("<span class='mathy-text'>#</span>", null);
  }

  runInputFlow(manager: AbstractButtonManager): Promise<void> {
    return numericalInputToStack(manager.inputManager, "");
  }
}

export class AlgebraicInputButton extends InputButton {
  constructor() {
    super("<math><mi>f</mi></math>", "'");
  }

  runInputFlow(manager: AbstractButtonManager): Promise<void> {
    return algebraicInputToStack(manager.inputManager, "");
  }
}

export class StringInputButton extends InputButton {
  constructor() {
    super("&quot;", '"');
  }

  runInputFlow(manager: AbstractButtonManager): Promise<void> {
    return stringInputToStack(manager.inputManager, "");
  }
}

// The flow for this button is sort of unique, since we have to get
// stack data (using the prefix arg), then get user input, and finally
// run a command. So the handling of the modifiers is done in the
// frontend class for this particular command. Most commands do this
// in the backend.
export class AlgebraicEditButton extends InputButton {
  constructor() {
    super(pencilSvg(), "`");
  }

  runInputFlow(manager: AbstractButtonManager): Promise<void> {
    const modifiers = manager.getModifiers();
    const stackIndex = Math.max(modifiers.prefixArgument ?? 1, 1) - 1;
    return editStackFrame(manager.inputManager, stackIndex, {
      isMouseInteraction: false,
      commandOptionsOverride: modifiersToRustArgs(modifiers),
    });
  }
}
