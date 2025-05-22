
import { AbstractButtonManager, ButtonGrid, GridCell } from "../button_grid.js";
import { Button, backButton, DispatchButton } from './button.js';
import { SignedNumberedButton } from './button/numbered.js';
import { SubcommandBehavior } from './subcommand.js';

export class FormulaButtonGrid extends ButtonGrid {
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
        new DispatchButton("=", "=", "="),
        new DispatchButton("≠", "!=", "+"),
        new DispatchButton("1<sup>st</sup>", "fhead", "h"),
        new FunctorArgsButton(),
      ],
      [
        new DispatchButton("<", "<", ","),
        new DispatchButton("≤", "<=", "<"),
        new DispatchButton("f()", "fcompile", null),
      ],
      [
        new DispatchButton(">", ">", "."),
        new DispatchButton("≥", ">=", ">"),
      ],
      [
        new DispatchButton("..", "..", null),
        new DispatchButton("..^", "..^", null),
        new DispatchButton("^..", "^..", null),
        new DispatchButton("^..^", "^..^", null),
      ],
      [
        new DispatchButton("<math><mi>∞</mi></math>", "infinity", null),
        new DispatchButton("<small><math><mo>-</mo><mi>∞</mi></math></small>", "neg_infinity", null),
        new DispatchButton("<math><mover><mrow><mi>∞</mi></mrow><mo>~</mo></mover></math>", "undir_infinity", null),
        new DispatchButton("nan", "nan_infinity", null),
      ],
      [
        backButton(this.rootGrid),
        new DispatchButton("π", "pi", null),
        new DispatchButton("e", "e", null),
        new DispatchButton("γ", "gamma", null),
        new DispatchButton("ϕ", "phi", null),
      ],
    ];
  }
}

// Delegates to SignedNumberedButton without the hyper flag, or simple
// DispatchButton with the hyper flag.
class FunctorArgsButton extends Button {
  private defaultButton: Button;
  private hyperButton: Button;

  constructor() {
    super("y<sup>th</sup>", "r");
    this.defaultButton = new SignedNumberedButton(this.label, "fargs", this.keyboardShortcut, "Index:");
    this.hyperButton = new DispatchButton(this.label, "fargs", this.keyboardShortcut);
  }

  async fire(manager: AbstractButtonManager) {
    if (manager.getModifiers().hyperbolicModifier) {
      await this.hyperButton.fire(manager);
    } else {
      await this.defaultButton.fire(manager);
    }
  }

  asSubcommand(): SubcommandBehavior {
    return "invalid";
  }
}
