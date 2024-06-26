
import { ButtonGridManager, ButtonGrid, GridCell } from "../button_grid.js";
import { AlgebraButtonGrid } from "./algebra_button_grid.js";
import { StorageButtonGrid } from "./storage_button_grid.js";
import { VectorButtonGrid } from "./vector_button_grid.js";
import { FormulaButtonGrid } from "./formula_button_grid.js";
import { TranscendentalButtonGrid } from "./transcendental_button_grid.js";
import { GraphingButtonGrid } from "./graphing_button_grid.js";
import { StringButtonGrid } from "./string_button_grid.js";
import { DisplayButtonGrid } from "./display_button_grid.js";
import { DispatchButton, GotoButton } from './button.js';
import { NumericalInputButton, AlgebraicInputButton,
         StringInputButton, AlgebraicEditButton } from './button/input.js';
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

function graphSvg(): HTMLElement {
  return svg('assets/graph.svg', {alt: "graph"});
}

export class MainButtonGrid extends ButtonGrid {
  private static NUMERICAL_INPUT_START_KEYS = new Set([
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", ".", "e", "_",
  ]);

  readonly rows: readonly (readonly GridCell[])[];

  private inputManager: InputBoxManager;
  private onEscapeDismissable: Hideable;

  private subgrids: Subgrids;

  constructor(inputManager: InputBoxManager, onEscapeDismissable: Hideable) {
    super();
    this.inputManager = inputManager;
    this.onEscapeDismissable = onEscapeDismissable;
    this.subgrids = new Subgrids(this, inputManager);
    this.rows = this.initRows();
  }

  private initRows(): GridCell[][] {
    return [
      [
        new DispatchButton("+", "+", "+"),
        new NumericalInputButton(this.inputManager),
        new AlgebraicInputButton(this.inputManager),
        new StringInputButton(this.inputManager),
        new AlgebraicEditButton(this.inputManager),
      ],
      [
        new DispatchButton("-", "-", "-"),
        new DispatchButton("<math><mo fence='true'>|</mo><mo>·</mo><mo fence='true'>|</mo></math>", "abs", "A"),
      ],
      [
        new DispatchButton("<math><mo>&times;</mo></math>", "*", "*"),
        new DispatchButton("<math><mo>&times;</mo><mi>i</mi></math>", "*i", null),
        new DispatchButton("<math><mo>&plusmn;</mo></math>", "negate", "n"),
        new DispatchButton("<math><msup><mi>x</mi><mi>y</mi></msup></math>", "^", "^"),
      ],
      [
        new DispatchButton("&divide;", "/", "/"),
        new DispatchButton("%", "%", "%"),
        new DispatchButton("&lfloor;&divide;&rfloor;", "div", "\\"),
        new DispatchButton("<span class='mathy-text'>x=</span>", "substitute_vars", "="),
        new GotoButton("str", null, this.subgrids.strings),
      ],
      [
        new DispatchButton(discardSvg(), "pop", "Backspace"),
        new DispatchButton(swapSvg(), "swap", "Tab"),
        new DispatchButton(dupSvg(), "dup", "Enter"),
        new GotoButton("<math><mi>ξ</mi></math>", "f", this.subgrids.transcendental),
        new GotoButton("out", "d", this.subgrids.display),
      ],
      [
        new GotoButton("<math><mi>x</mi></math>", "a", this.subgrids.algebra),
        new GotoButton(":=", "s", this.subgrids.storage),
        new GotoButton("[]", "v", this.subgrids.vector),
        new GotoButton("≤", null, this.subgrids.formula),
        new GotoButton(graphSvg(), "g", this.subgrids.graphing),
      ],
    ];
  }

  async onUnhandledKey(input: KeyEventInput, manager: ButtonGridManager): Promise<KeyResponse> {
    const key = input.toEmacsSyntax();

    if (["L", "B", "E", "G", "J"].includes(key)) {
      const transcendentalTable = this.subgrids.transcendental.getKeyMappingTable();
      await transcendentalTable[key].fire(manager);
      return KeyResponse.BLOCK;
    }

    if (["M-u", "M-l"].includes(key)) {
      const stringTable = this.subgrids.strings.getKeyMappingTable();
      await stringTable[key].fire(manager);
      return KeyResponse.BLOCK;
    }

    // Emacs compatibility: Forward "|" to vector button grid.
    if (key == "|") {
      const vectorTable = this.subgrids.vector.getKeyMappingTable();
      await vectorTable[key].fire(manager);
      return KeyResponse.BLOCK;
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
    const transcendentalTable = this.subgrids.transcendental.getKeyMappingTable();
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

class Subgrids {
  readonly algebra: AlgebraButtonGrid;
  readonly formula: FormulaButtonGrid;
  readonly storage: StorageButtonGrid;
  readonly vector: VectorButtonGrid;
  readonly transcendental: TranscendentalButtonGrid;
  readonly graphing: GraphingButtonGrid;
  readonly display: DisplayButtonGrid;
  readonly strings: StringButtonGrid;

  constructor(mainGrid: MainButtonGrid, inputManager: InputBoxManager) {
    this.algebra = new AlgebraButtonGrid(mainGrid, inputManager);
    this.formula = new FormulaButtonGrid(mainGrid, inputManager);
    this.storage = new StorageButtonGrid(mainGrid, inputManager);
    this.vector = new VectorButtonGrid(mainGrid, inputManager);
    this.transcendental = new TranscendentalButtonGrid(mainGrid, inputManager);
    this.graphing = new GraphingButtonGrid(mainGrid, inputManager);
    this.display = new DisplayButtonGrid(mainGrid, inputManager);
    this.strings = new StringButtonGrid(mainGrid, inputManager);
  }
}
