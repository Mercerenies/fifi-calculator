
import { ButtonGrid, GridCell } from "../button_grid.js";
import { backButton, DispatchButton } from './button.js';
import { InputBoxManager } from '../input_box.js';

export class FormulaButtonGrid extends ButtonGrid {
  readonly rows: readonly (readonly GridCell[])[];

  private rootGrid: ButtonGrid;
  private inputManager: InputBoxManager;

  constructor(rootGrid: ButtonGrid, inputManager: InputBoxManager) {
    super();
    this.rootGrid = rootGrid;
    this.inputManager = inputManager;
    this.rows = this.initRows();
  }

  private initRows(): GridCell[][] {
    return [
      [
        new DispatchButton("=", "=", "="),
        new DispatchButton("≠", "!=", "+"),
      ],
      [
        new DispatchButton("<", "<", ","),
        new DispatchButton("≤", "<=", "<"),
      ],
      [
        new DispatchButton(">", ">", "."),
        new DispatchButton("≥", ">=", ">"),
      ],
      [
        new DispatchButton("..", "..", null),
        new DispatchButton("..^", "..^", null),
        new DispatchButton("^..", "^..", null),
        new DispatchButton("^..^", "^..^", null),
      ],
      [
        new DispatchButton("<math><mi>∞</mi></math>", "infinity", null),
        new DispatchButton("<small><math><mo>-</mo><mi>∞</mi></math></small>", "neg_infinity", null),
        new DispatchButton("<math><mover><mrow><mi>∞</mi></mrow><mo>~</mo></mover></math>", "undir_infinity", null),
        new DispatchButton("nan", "nan_infinity", null),
      ],
      [
        backButton(this.rootGrid),
        new DispatchButton("π", "pi", null),
        new DispatchButton("e", "e", null),
        new DispatchButton("γ", "gamma", null),
        new DispatchButton("ϕ", "phi", null),
      ],
    ];
  }
}
