
import * as Page from './page.js';
import { InputBoxManager } from './input_box.js';
import { NotificationManager } from './notifications.js';
import { MainButtonGrid } from './button_grid/main_button_grid.js';
import { KeyInput } from './keyboard.js';
import * as KeyDispatcher from './keyboard/dispatcher.js';
import { RightPanelManager } from './right_panel.js';
import { TAURI, RefreshStackPayload, UndoAvailabilityPayload } from './tauri_api.js';
import { StackView } from './stack_view.js';
import { GraphicsEngine } from './graphics_engine.js';

import { OsType } from '@tauri-apps/plugin-os';

class UiManager {
  readonly inputManager: InputBoxManager;
  readonly notificationManager: NotificationManager;
  readonly rightPanelManager: RightPanelManager;
  readonly osType: OsType;

  private keyHandler: KeyDispatcher.KeyEventHandler;

  constructor(osType: OsType) {
    this.inputManager = new InputBoxManager({
      inputBox: Page.getInputBoxDiv(),
      inputTextBox: Page.getInputTextBox(),
      inputLabel: Page.getInputTextBoxLabel(),
    });
    this.notificationManager = new NotificationManager(Page.getNotificationBox());
    this.rightPanelManager = new RightPanelManager({
      buttonGrid: Page.getButtonGridContainer(),
      prefixPanel: Page.getPrefixArgPanel(),
      keepModifierCheckbox: Page.getModifierArgKeepArgCheckbox(),
      initialGrid: new MainButtonGrid(this.inputManager, this.notificationManager),
      undoButton: Page.getUndoButton(),
      redoButton: Page.getRedoButton(),
    });
    this.osType = osType;

    this.keyHandler = KeyDispatcher.sequential([
      KeyDispatcher.filtered(
        this.inputManager,
        () => document.activeElement === Page.getInputTextBox(),
      ),
      this.rightPanelManager.undoManager,
      this.rightPanelManager.buttonGrid,
    ]);
  }

  static async create(): Promise<UiManager> {
    const osType = await TAURI.osType();
    return new UiManager(osType);
  }

  initListeners(): void {
    this.inputManager.initListeners();
    this.notificationManager.initListeners();
    this.rightPanelManager.initListeners();
    document.body.addEventListener("keydown", (event) => this.dispatchOnKey(event));
  }

  private async dispatchOnKey(event: KeyboardEvent): Promise<void> {
    const input = KeyInput.fromEvent(event, this.osType);
    if (input === undefined) {
      // The pressed key was a modifier key like Ctrl or Alt, so
      // ignore it.
      return;
    }

    this.keyHandler.onKeyDown(input);
  }
}

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

window.addEventListener("DOMContentLoaded", async function() {
  const graphicsEngine = new GraphicsEngine();
  const uiManager = await UiManager.create();
  const stackView = new StackView(Page.getValueStack(), graphicsEngine);
  uiManager.initListeners();
  await TAURI.listen("refresh-stack", (event) => refreshStack(stackView, event.payload));
  await TAURI.listen("show-error", (event) => uiManager.notificationManager.show(event.payload.errorMessage));
  await TAURI.listen("refresh-undo-availability", (event) => refreshUndoButtons(uiManager, event.payload));
});
