
import { KeyEventInput, KeyResponse } from './keyboard.js';
import { TAURI } from './tauri_api.js';

export class UndoManager {
  private undoButton: HTMLButtonElement;
  private redoButton: HTMLButtonElement;

  constructor(args: UndoManagerArguments) {
    this.undoButton = args.undoButton;
    this.redoButton = args.redoButton;
  }

  initListeners(): void {
    this.undoButton.addEventListener("click", () =>
      this.doUndoAction("undo"));

    this.redoButton.addEventListener("click", () =>
      this.doUndoAction("redo"));
  }

  private doUndoAction(direction: "undo" | "redo"): Promise<void> {
    return TAURI.performUndoAction(direction);
  }

  setUndoButtonEnabled(enabled: boolean): void {
    this.undoButton.disabled = !enabled;
  }

  setRedoButtonEnabled(enabled: boolean): void {
    this.redoButton.disabled = !enabled;
  }

  async onKeyDown(input: KeyEventInput): Promise<KeyResponse> {
    switch (input.toEmacsSyntax()) {
      case "U":
      case "C-/":
        this.doUndoAction("undo");
        return KeyResponse.BLOCK;
      case "D":
        this.doUndoAction("redo");
        return KeyResponse.BLOCK;
      default:
        return KeyResponse.PASS;
    }
  }
}

export interface UndoManagerArguments {
  undoButton: HTMLButtonElement;
  redoButton: HTMLButtonElement;
}
