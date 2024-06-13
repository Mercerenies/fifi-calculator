
import { ButtonGridManager, ButtonGrid, GridCell } from "../button_grid.js";
import { AlgebraButtonGrid } from "./algebra_button_grid.js";
import { StorageButtonGrid } from "./storage_button_grid.js";
import { VectorButtonGrid } from "./vector_button_grid.js";
import { TranscendentalButtonGrid } from "./transcendental_button_grid.js";
import { DispatchButton, GotoButton } from './button.js';
import { NumericalInputButton, AlgebraicInputButton } from './button/input.js';
import { InputBoxManager } from '../input_box.js';
import { numericalInputToStack } from '../input_box/numerical_input.js';
import { KeyEventInput, KeyResponse } from '../keyboard.js';
import { svg } from '../util.js';

export interface Hideable {
  hide(): void;
}

function discardSvg(): HTMLElement {
  return svg('assets/discard.svg', {alt: "pop"});
}

function swapSvg(): HTMLElement {
  return svg('assets/swap.svg', {alt: "swap"});
}

function dupSvg(): HTMLElement {
  return svg('assets/duplicate.svg', {alt: "dup"});
}

export class MainButtonGrid extends ButtonGrid {
  private static NUMERICAL_INPUT_START_KEYS = new Set([
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", ".", "e", "_",
  ]);

  readonly rows: readonly (readonly GridCell[])[];

  private inputManager: InputBoxManager;
  private onEscapeDismissable: Hideable;

  // We store this one, since we delegate keys to it, and we don't
  // want to have to regenerate it every time a key is pressed.
  private transcendentalButtonGrid: TranscendentalButtonGrid;

  constructor(inputManager: InputBoxManager, onEscapeDismissable: Hideable) {
    super();
    this.inputManager = inputManager;
    this.onEscapeDismissable = onEscapeDismissable;
    this.transcendentalButtonGrid = new TranscendentalButtonGrid(this, inputManager);
    this.rows = this.initRows();
  }

  private initRows(): GridCell[][] {
    return [
      [
        new DispatchButton("+", "+", "+"),
      ],
      [
        new DispatchButton("-", "-", "-"),
      ],
      [
        new DispatchButton("<math><mo>&times;</mo></math>", "*", "*"),
        new DispatchButton("<math><mo>&times;</mo><mi>i</mi></math>", "*i", null),
        new DispatchButton("<math><mo>&plusmn;</mo></math>", "negate", "n"),
        new DispatchButton("<math><msup><mi>x</mi><mi>y</mi></msup></math>", "^", "^"),
        new GotoButton("<math><mi>ξ</mi></math>", null, this.transcendentalButtonGrid),
      ],
      [
        new DispatchButton("&divide;", "/", "/"),
        new DispatchButton("%", "%", "%"),
        new DispatchButton("&lfloor;&divide;&rfloor;", "div", "\\"),
        new DispatchButton("=", "substitute_vars", "="),
      ],
      [
        new DispatchButton(discardSvg(), "pop", "Backspace"),
        new DispatchButton(swapSvg(), "swap", "Tab"),
        new DispatchButton(dupSvg(), "dup", "Enter"),
        new NumericalInputButton(this.inputManager),
        new AlgebraicInputButton(this.inputManager),
      ],
      [
        new GotoButton("<math><mi>x</mi></math>", "a", () => new AlgebraButtonGrid(this, this.inputManager)),
        new GotoButton(":=", "s", () => new StorageButtonGrid(this, this.inputManager)),
        new GotoButton("[]", "v", () => new VectorButtonGrid(this, this.inputManager)),
      ],
    ];
  }

  async onUnhandledKey(input: KeyEventInput, manager: ButtonGridManager): Promise<KeyResponse> {
    const key = input.toEmacsSyntax();

    const transcendentalResult = await this.tryDelegateToTranscendentalGrid(manager, key);
    if (transcendentalResult !== null) {
      return transcendentalResult;
    }

    if (MainButtonGrid.NUMERICAL_INPUT_START_KEYS.has(key)) {
      // Start numerical input
      input.event.preventDefault();
      numericalInputToStack(this.inputManager, this.translateInitialInput(key)); // Fire-and-forget promise
      return KeyResponse.BLOCK;
    } else if (key === "Escape") {
      this.onEscapeDismissable.hide();
      return KeyResponse.BLOCK;
    } else {
      return KeyResponse.PASS;
    }
  }

  // Returns null if not handled, or a KeyResponse if handled.
  private async tryDelegateToTranscendentalGrid(manager: ButtonGridManager, key: string): Promise<KeyResponse | null> {
    const transcendentalTable = this.transcendentalButtonGrid.getKeyMappingTable();
    /* eslint-disable-next-line @typescript-eslint/no-dynamic-delete */
    delete transcendentalTable["Escape"]; // Don't forward "Escape", which just doubles back to this grid.
    if (key in transcendentalTable) {
      await transcendentalTable[key].fire(manager);
      return KeyResponse.BLOCK;
    } else {
      return null;
    }
  }

  private translateInitialInput(key: string): string {
    switch (key) {
    case "e":
      return "1e";
    case "_":
      return "-";
    default:
      return key;
    }
  }
}
