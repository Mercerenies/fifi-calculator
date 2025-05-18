
import { ButtonGrid, GridCell } from "../button_grid.js";
import { backButton, DispatchButton } from './button.js';

export class DatetimeButtonGrid extends ButtonGrid {
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
        new DispatchButton("<span class='mathy-text'>-0</span>", "days_since_zero", "D"),
        new DispatchButton("J", "julian_day", "J"),
      ],
      [
      ],
      [
      ],
      [
      ],
      [
      ],
      [
        backButton(this.rootGrid),
      ],
    ];
  }
}
