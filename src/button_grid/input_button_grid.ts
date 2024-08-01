
import { ButtonGrid, GridCell } from "../button_grid.js";
import { backButton, DispatchButton } from './button.js';
import { SignedNumberedButton, UnsignedNumberedButton } from './button/numbered.js';
import { StringInputButton, AlgebraicEditButton } from './button/input.js';

export class InputButtonGrid extends ButtonGrid {
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
        new StringInputButton(),
        new AlgebraicEditButton(),
      ],
      [
        new DispatchButton("[", "incomplete[", "["),
        new DispatchButton("]", "incomplete]", "]"),
      ],
      [
        new DispatchButton("(", "incomplete(", "("),
        new DispatchButton(")", "incomplete)", ")"),
      ],
      [],
      [],
      [
        backButton(this.rootGrid),
      ],
    ];
  }
}
