
import { ButtonGrid, GridCell } from "../button_grid.js";
import { backButton, DispatchButton } from './button.js';
import { SignedNumberedButton, UnsignedNumberedButton } from './button/numbered.js';

export class MatrixButtonGrid extends ButtonGrid {
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
        new DispatchButton("1<sup>st</sup>", "head", "h"),
        new SignedNumberedButton("y<sup>th</sup>", "nth", "r", "Index:"),
        new SignedNumberedButton("x<sup>th</sup>", "nth_column", "c", "Index:"),
      ],
      [
        new DispatchButton("↘", "diag", "d"),
        new UnsignedNumberedButton("<math><msub><mi>I</mi><mi>n</mi></msub></math>", "identity_matrix", "i", "Dims:"),
        new DispatchButton("<math><msup><mi>A</mi><mi>T</mi></msup></math>", "transpose", "t"),
      ],
      [
        new DispatchButton("<math><msup><mi>A</mi><mn>-1</mn></msup></math>", "recip", "&"),
      ],
      [],
      [],
      [
        backButton(this.rootGrid),
      ],
    ];
  }
}
