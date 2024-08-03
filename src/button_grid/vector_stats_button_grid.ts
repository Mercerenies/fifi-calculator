
import { ButtonGrid, GridCell } from "../button_grid.js";
import { backButton, DispatchButton } from './button.js';

export class VectorStatsButtonGrid extends ButtonGrid {
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
        new DispatchButton("<math><mover><mrow><mi>x</mi></mrow><mo>-</mo></mover></math>", "mean", "M"),
        new DispatchButton("H", "hmean", "H"),
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
