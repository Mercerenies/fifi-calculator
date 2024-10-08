
import { ButtonGrid, GridCell } from "../button_grid.js";
import { backButton, DispatchButton } from './button.js';

export class TranscendentalButtonGrid extends ButtonGrid {
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
        new DispatchButton("ln", "ln", "L"),
        new DispatchButton("log", "log", "B"),
      ],
      [
        new DispatchButton("<math><msup><mi>e</mi><mi>x</mi></msup></math>", "e^", "E"),
        new DispatchButton("<math><msqrt><mi>x</mi></msqrt></math>", "sqrt", "Q"),
        new DispatchButton("N", "substitute_numerically", "N"),
      ],
      [
        new DispatchButton("<math><mover><mi>z</mi><mo>-</mo></mover></math>", "conj", "J"),
        new DispatchButton("sgn", "signum", "s"),
        new DispatchButton("arg", "arg", "G"),
        new DispatchButton("re", "re", "r"),
        new DispatchButton("im", "im", "i"),
      ],
      [
        new DispatchButton("<small><math><mrow><mo>⌊</mo><mi>·</mi><mo>⌋</mo></mrow></math></small>", "min", "n"),
        new DispatchButton("<small><math><mrow><mo>⌈</mo><mi>·</mi><mo>⌉</mo></mrow></math></small>", "max", "x"),
      ],
      [
        new DispatchButton("sin", "sin", "S"),
        new DispatchButton("cos", "cos", "C"),
        new DispatchButton("tan", "tan", "T"),
      ],
      [
        backButton(this.rootGrid),
      ],
    ];
  }
}
