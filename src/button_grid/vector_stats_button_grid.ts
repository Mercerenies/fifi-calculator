
import { ButtonGrid, GridCell } from "../button_grid.js";
import { backButton, DispatchButton } from './button.js';
import { UnsignedNumberedButton } from './button/numbered.js';
import { svg } from '../util.js';

function magnifyingLensSvg(): HTMLElement {
  return svg('assets/magnifying.svg', {alt: "search"});
}

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
