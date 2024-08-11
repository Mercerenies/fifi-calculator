
import * as Page from './page.js';
import { UiManager } from './ui_manager.js';
import { TAURI, RefreshStackPayload, UndoAvailabilityPayload,
         ModelinePayload, defaultCommandOptions } from './tauri_api.js';
import { StackView, StackUpdatedDelegate } from './stack_view.js';
import { GRAPHICS_DELEGATE } from './graphics.js';

async function refreshStack(stackView: StackView, payload: RefreshStackPayload): Promise<void> {
  await stackView.refreshStack(payload.stack);
  if (payload.forceScrollDown) {
    stackView.scrollToBottom();
  }
}

function refreshUndoButtons(uiManager: UiManager, state: UndoAvailabilityPayload) {
  const undoManager = uiManager.rightPanelManager.undoManager;
  undoManager.setUndoButtonEnabled(state.hasUndos);
  undoManager.setRedoButtonEnabled(state.hasRedos);
}

function refreshModeline(payload: ModelinePayload) {
  const modeline = Page.getModelineBar();
  const pre = document.createElement("pre");
  pre.innerHTML = payload.modelineText;
  modeline.innerHTML = "";
  modeline.appendChild(pre);
}

window.addEventListener("DOMContentLoaded", async function() {
  const uiManager = await UiManager.create();
  const stackView = new StackView(
    Page.getValueStack(),
    StackUpdatedDelegate.several([
      GRAPHICS_DELEGATE,
      uiManager.rightPanelManager.touchModeManager,
    ]),
  );

  uiManager.initListeners();
  await TAURI.listen("refresh-stack", (event) => refreshStack(stackView, event.payload));
  await TAURI.listen("show-error", (event) => uiManager.notificationManager.show(event.payload.errorMessage));
  await TAURI.listen("refresh-undo-availability", (event) => refreshUndoButtons(uiManager, event.payload));
  await TAURI.listen("refresh-modeline", (event) => refreshModeline(event.payload));

  // Send a nop command, just to flush the stack and undo buttons in
  // case we were resumed from a paused state.
  await TAURI.runMathCommand("nop", [], defaultCommandOptions());
});

