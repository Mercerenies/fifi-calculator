
import { ButtonGrid, GridCell } from "../button_grid.js";
import { backButton, DispatchButton } from './button.js';
import * as HelpLibrary from "../help_library.js";

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
        new DispatchButton("ln", "ln", "L", HelpLibrary.ln),
        new DispatchButton("log", "log", "B", HelpLibrary.log),
      ],
      [
        new DispatchButton("<math><msup><mi>e</mi><mi>x</mi></msup></math>", "e^", "E", HelpLibrary.exp),
        new DispatchButton("<math><msqrt><mi>x</mi></msqrt></math>", "sqrt", "Q", HelpLibrary.sqrt),
        new DispatchButton("N", "substitute_numerically", "N", HelpLibrary.substituteNumerically),
      ],
      [
        new DispatchButton("<math><mover><mi>z</mi><mo>-</mo></mover></math>", "conj", "J", HelpLibrary.conjugate),
        new DispatchButton("sgn", "signum", "s", HelpLibrary.signum),
        new DispatchButton("arg", "arg", "G", HelpLibrary.complexArg),
        new DispatchButton("re", "re", "r", HelpLibrary.realPart),
        new DispatchButton("im", "im", "i", HelpLibrary.imagPart),
      ],
      [
        new DispatchButton("<small><math><mrow><mo>⌊</mo><mi>·</mi><mo>⌋</mo></mrow></math></small>", "min", "n", HelpLibrary.minFunction),
        new DispatchButton("<small><math><mrow><mo>⌈</mo><mi>·</mi><mo>⌉</mo></mrow></math></small>", "max", "x", HelpLibrary.maxFunction),
      ],
      [
        new DispatchButton("sin", "sin", "S", HelpLibrary.sine),
        new DispatchButton("cos", "cos", "C", HelpLibrary.cosine),
        new DispatchButton("tan", "tan", "T", HelpLibrary.tangent),
      ],
      [
        backButton(this.rootGrid),
      ],
    ];
  }
}
