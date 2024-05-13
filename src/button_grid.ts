
// Manager class for the button grid that shows up on-screen and for
// keyboard shortcuts to said grid.

// NOTE: This should be kept up to date with the .button-grid class in
// styles.css. If that value gets updated, update this as well!
const GRID_CELLS_PER_ROW = 5;

// This one doesn't appear in the CSS; it just determines how many
// nodes we generate.
const GRID_ROWS = 6;

export class ButtonGridManager {
  private domElement: HTMLElement;
  private activeGrid: ButtonGrid;
  private buttonsByKey: Record<string, GridCell> = {};

  constructor(domElement: HTMLElement, initialGrid: ButtonGrid) {
    this.domElement = domElement;
    this.activeGrid = initialGrid;
    this.setActiveGrid(initialGrid); // Initialize the grid
  }

  setActiveGrid(grid: ButtonGrid): void {
    this.activeGrid = grid;
    this.loadButtonShortcuts();
    this.loadHtml();
  }

  private loadButtonShortcuts(): void {
    this.buttonsByKey = {};
    for (const row of this.activeGrid.rows) {
      for (const button of row) {
        const keyboardShortcut = button.keyboardShortcut;
        if (keyboardShortcut !== null) {
          if (keyboardShortcut in this.buttonsByKey) {
            console.warn("Duplicate keyboard shortcut in grid:", keyboardShortcut);
          }
          this.buttonsByKey[keyboardShortcut] = button;
        }
      }
    }
  }

  private loadHtml(): void {
    this.domElement.innerHTML = "";
    const gridDiv = document.createElement("div");
    gridDiv.classList.add("button-grid");
    for (let y = 0; y < GRID_ROWS; y++) {
      const row = this.activeGrid.rows[y] ?? [];
      for (let x = 0; x < GRID_CELLS_PER_ROW; x++) {
        const gridCell = row[x] ?? new Spacer();
        gridDiv.appendChild(gridCell.getHTML(this));
      }
    }
    this.domElement.appendChild(gridDiv);
  }

  async onKeyDown(event: KeyboardEvent): Promise<void> {
    const button = this.buttonsByKey[event.key];
    if (button !== undefined) {
      event.preventDefault();
      await button.fire(this);
    } else {
      await this.activeGrid.onUnhandledKey(event);
    }
  }
}

export interface ButtonGrid {
  // Should be at most a GRID_ROWS * GRID_CELLS_BY_ROW array. If this
  // grid is smaller than that size, the missing elements will be
  // filled in with Spacer objects.
  readonly rows: readonly (readonly GridCell[])[];

  onUnhandledKey(event: KeyboardEvent): Promise<void>;
}

export interface GridCell {
  readonly keyboardShortcut: string | null;

  getHTML(manager: ButtonGridManager): HTMLElement;
  fire(manager: ButtonGridManager): Promise<void>;
}

// Empty grid cell.
export class Spacer implements GridCell {
  readonly keyboardShortcut: string | null = null;

  getHTML(): HTMLElement {
    return document.createElement("div");
  }

  fire(): Promise<void> {
    // No action.
    return Promise.resolve();
  }
}
