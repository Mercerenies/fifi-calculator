
import { ButtonGrid, GridCell } from "../button_grid.js";
import { backButton, DispatchButton } from './button.js';

export class StringButtonGrid extends ButtonGrid {
  readonly rows: readonly (readonly GridCell[])[];

  private rootGrid: ButtonGrid;

  constructor(rootGrid: ButtonGrid) {
    super();
    this.rootGrid = rootGrid;
    this.rows = this.initRows();
  }

  private initRows(): GridCell[][] {
    return [
      [
        new DispatchButton("AZ", "uppercase", "M-u"),
        new DispatchButton("az", "lowercase", "M-l"),
      ],
      [],
      [],
      [],
      [],
      [
        backButton(this.rootGrid),
      ],
    ];
  }
}
