
import { ButtonGrid, GridCell } from "../button_grid.js";
import { backButton, DispatchButton } from './button.js';
import { InputBoxManager } from '../input_box.js';

export class TranscendentalButtonGrid extends ButtonGrid {
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
        new DispatchButton("ln", "ln", "L"),
        new DispatchButton("log", "log", "B"),
      ],
      [
        new DispatchButton("<math><msup><mi>e</mi><mi>x</mi></msup></math>", "e^", "E"),
      ],
      [
        new DispatchButton("<math><mover><mi>z</mi><mo>-</mo></mover></math>", "conj", "J"),
        new DispatchButton("sgn", "signum", "s"),
        new DispatchButton("arg", "arg", "G"),
      ],
      [
        new DispatchButton("re", "re", "r"),
        new DispatchButton("im", "im", "i"),
      ],
      [],
      [
        backButton(this.rootGrid),
      ],
    ];
  }
}
