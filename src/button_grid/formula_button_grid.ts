
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
      [],
      [
        backButton(this.rootGrid),
      ],
    ];
  }
}
