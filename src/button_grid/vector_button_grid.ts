
import { ButtonGrid, GridCell } from "../button_grid.js";
import { backButton, DispatchButton } from './button.js';
import { InputBoxManager } from '../input_box.js';

export class VectorButtonGrid extends ButtonGrid {
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
        new DispatchButton("p", "pack", "p"),
        new DispatchButton("u", "unpack", "u"),
        new DispatchButton("<math><mi>ι</mi></math>", "iota", "x"),
        new DispatchButton("<math><mo>*</mo></math>", "repeat", "b"),
      ],
      [
        new DispatchButton("++", "vconcat", "|"),
        new DispatchButton("::", "cons", "k"),
      ],
      [
        new DispatchButton("1<sup>st</sup>", "head", "h"),
      ],
      [
        new DispatchButton("[", "incomplete[", "["),
      ],
      [
        new DispatchButton("(", "incomplete(", "("),
      ],
      [
        backButton(this.rootGrid),
      ],
    ];
  }
}
