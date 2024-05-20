
const tauri = window.__TAURI__.tauri;
const events = window.__TAURI__.events;

export class UndoManager {
  private undoButton: HTMLButtonElement;
  private redoButton: HTMLButtonElement;

  constructor(args: UndoManagerArguments) {
    this.undoButton = args.undoButton;
    this.redoButton = args.redoButton;
  }

  initListeners(): void {
    this.undoButton.addEventListener("click", () =>
      tauri.invoke('perform_undo_action', { direction: "undo" }));
    this.redoButton.addEventListener("click", () =>
      tauri.invoke('perform_undo_action', { direction: "redo" }));
  }

  setUndoButtonEnabled(enabled: boolean): void {
    this.undoButton.disabled = !enabled;
  }

  setRedoButtonEnabled(enabled: boolean): void {
    this.redoButton.disabled = !enabled;
  }
}

export interface UndoManagerArguments {
  undoButton: HTMLButtonElement;
  redoButton: HTMLButtonElement;
}
