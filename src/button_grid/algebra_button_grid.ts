
import { ButtonGrid, GridCell } from "../button_grid.js";
import { KeyResponse } from '../keyboard.js';

export class AlgebraButtonGrid implements ButtonGrid {
  readonly rows: readonly (readonly GridCell[])[];

  private rootGrid: ButtonGrid;

  constructor(rootGrid: ButtonGrid) {
    this.rootGrid = rootGrid;
    this.rows = this.initRows();
  }

  private initRows(): GridCell[][] {
    return [
      [],
      [],
      [],
      [],
      [],
      [],
    ];
  }

  onUnhandledKey(): Promise<KeyResponse> {
    return Promise.resolve(KeyResponse.PASS);
  }
}
