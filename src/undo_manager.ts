
export class UndoManager {
  private undoButton: HTMLButtonElement;
  private redoButton: HTMLButtonElement;

  constructor(args: UndoManagerArgs) {
    this.undoButton = args.undoButton;
    this.redoButton = args.redoButton;
  }

  initListeners(): void {
    // TODO
  }
}

export interface UndoManagerArgs {
  undoButton: HTMLButtonElement;
  redoButton: HTMLButtonElement;
}
