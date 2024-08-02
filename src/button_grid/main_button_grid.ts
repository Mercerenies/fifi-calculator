
import { ButtonGridManager, ButtonGrid, GridCell } from "../button_grid.js";
import { AlgebraButtonGrid } from "./algebra_button_grid.js";
import { StorageButtonGrid } from "./storage_button_grid.js";
import { VectorButtonGrid } from "./vector_button_grid.js";
import { MatrixButtonGrid } from "./matrix_button_grid.js";
import { FormulaButtonGrid } from "./formula_button_grid.js";
import { TranscendentalButtonGrid } from "./transcendental_button_grid.js";
import { GraphingButtonGrid } from "./graphing_button_grid.js";
import { StringButtonGrid } from "./string_button_grid.js";
import { DisplayButtonGrid } from "./display_button_grid.js";
import { ModesButtonGrid } from "./modes_button_grid.js";
import { UnitsButtonGrid } from "./units_button_grid.js";
import { InputButtonGrid } from "./input_button_grid.js";
import { DispatchButton, GotoButton } from './button.js';
import { NumericalInputButton, AlgebraicInputButton } from './button/input.js';
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

function rulerSvg(): HTMLElement {
  return svg('assets/ruler.svg', {alt: "units"});
}

export class MainButtonGrid extends ButtonGrid {
  private static NUMERICAL_INPUT_START_KEYS = new Set([
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", ".", "e", "_",
  ]);

  readonly rows: readonly (readonly GridCell[])[];

  private onEscapeDismissable: Hideable;

  private subgrids: Subgrids;

  constructor(onEscapeDismissable: Hideable) {
    super();
    this.onEscapeDismissable = onEscapeDismissable;
    this.subgrids = new Subgrids(this);
    this.rows = this.initRows();
  }

  private initRows(): GridCell[][] {
    return [
      [
        new DispatchButton("+", "+", "+"),
        new NumericalInputButton(),
        new AlgebraicInputButton(),
        new GotoButton("&quot;", null, this.subgrids.input),
      ],
      [
        new DispatchButton("-", "-", "-"),
        new DispatchButton("<math><mo fence='true'>|</mo><mo>·</mo><mo fence='true'>|</mo></math>", "abs", "A"),
        new GotoButton("mo", "m", this.subgrids.modes),
        new GotoButton("out", "d", this.subgrids.display),
        new GotoButton(graphSvg(), "g", this.subgrids.graphing),
      ],
      [
        new DispatchButton("<math><mo>&times;</mo></math>", "*", "*"),
        new DispatchButton("<math><mo>&times;</mo><mi>i</mi></math>", "*i", null),
        new DispatchButton("<math><mo>&plusmn;</mo></math>", "negate", "n"),
        new DispatchButton("<math><msup><mi>x</mi><mi>y</mi></msup></math>", "^", "^"),
        new GotoButton(rulerSvg(), "u", this.subgrids.units),
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
      ],
      [
        new GotoButton("<math><mi>x</mi></math>", "a", this.subgrids.algebra),
        new GotoButton(":=", "s", this.subgrids.storage),
        new GotoButton("[v]", "v", this.subgrids.vector),
        new GotoButton("[m]", "V", this.subgrids.matrix),
        new GotoButton("≤", null, this.subgrids.formula),
      ],
    ];
  }

  async onUnhandledKey(input: KeyEventInput, manager: ButtonGridManager): Promise<KeyResponse> {
    const key = input.toEmacsSyntax();

    const forwardingRule = SUBGRID_FORWARDING_TABLE[key];
    if (forwardingRule !== undefined) {
      const table = this.subgrids[forwardingRule].getKeyMappingTable();
      await table[key].fire(manager);
      return KeyResponse.BLOCK;
    }

    if (MainButtonGrid.NUMERICAL_INPUT_START_KEYS.has(key)) {
      // Start numerical input
      input.event.preventDefault();
      numericalInputToStack(manager.inputManager, this.translateInitialInput(key)); // Fire-and-forget promise
      return KeyResponse.BLOCK;
    } else if (key === "Escape") {
      this.onEscapeDismissable.hide();
      return KeyResponse.BLOCK;
    } else {
      return KeyResponse.PASS;
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
  readonly display: DisplayButtonGrid;
  readonly formula: FormulaButtonGrid;
  readonly graphing: GraphingButtonGrid;
  readonly input: InputButtonGrid;
  readonly matrix: MatrixButtonGrid;
  readonly modes: ModesButtonGrid;
  readonly storage: StorageButtonGrid;
  readonly strings: StringButtonGrid;
  readonly transcendental: TranscendentalButtonGrid;
  readonly units: UnitsButtonGrid;
  readonly vector: VectorButtonGrid;

  constructor(mainGrid: MainButtonGrid) {
    this.algebra = new AlgebraButtonGrid(mainGrid);
    this.display = new DisplayButtonGrid(mainGrid);
    this.formula = new FormulaButtonGrid(mainGrid);
    this.graphing = new GraphingButtonGrid(mainGrid);
    this.input = new InputButtonGrid(mainGrid);
    this.matrix = new MatrixButtonGrid(mainGrid);
    this.modes = new ModesButtonGrid(mainGrid);
    this.storage = new StorageButtonGrid(mainGrid);
    this.strings = new StringButtonGrid(mainGrid);
    this.transcendental = new TranscendentalButtonGrid(mainGrid);
    this.units = new UnitsButtonGrid(mainGrid);
    this.vector = new VectorButtonGrid(mainGrid);
  }
}

const SUBGRID_FORWARDING_TABLE: Record<string, keyof Subgrids> = {
  "L": "transcendental",
  "B": "transcendental",
  "S": "transcendental",
  "C": "transcendental",
  "T": "transcendental",
  "E": "transcendental",
  "Q": "transcendental",
  "J": "transcendental",
  "G": "transcendental",
  "M-u": "strings",
  "M-l": "strings",
  "[": "input",
  "]": "input",
  "(": "input",
  ")": "input",
  "\"": "input",
  "`": "input",
  "|": "vector",
  "&": "matrix",
  "@": "matrix",
};
