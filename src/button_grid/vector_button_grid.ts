
import { ButtonGrid, GridCell } from "../button_grid.js";
import { backButton, DispatchButton } from './button.js';
import { SignedNumberedButton, UnsignedNumberedButton } from './button/numbered.js';

export class VectorButtonGrid extends ButtonGrid {
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
        new DispatchButton("p", "pack", "p"),
        new DispatchButton("u", "unpack", "u"),
        new DispatchButton("<math><mi>ι</mi></math>", "iota", "x"),
        new DispatchButton("<math><mo>*</mo></math>", "repeat", "b"),
      ],
      [
        new DispatchButton("++", "vconcat", "|"),
        new DispatchButton("::", "cons", "k"),
        new DispatchButton("1<sup>st</sup>", "head", "h"),
        new SignedNumberedButton("y<sup>th</sup>", "nth", "r", "Index:"),
        new SignedNumberedButton("x<sup>th</sup>", "nth_column", "c", "Index:"),
      ],
      [
        new DispatchButton("↘", "diag", "d"),
        new UnsignedNumberedButton("<math><msub><mi>I</mi><mi>n</mi></msub></math>", "identity_matrix", "i", "Dims:"),
      ],
      [
        new DispatchButton("[", "incomplete[", "["),
        new DispatchButton("]", "incomplete]", "]"),
      ],
      [
        new DispatchButton("(", "incomplete(", "("),
        new DispatchButton(")", "incomplete)", ")"),
      ],
      [
        backButton(this.rootGrid),
      ],
    ];
  }
}
