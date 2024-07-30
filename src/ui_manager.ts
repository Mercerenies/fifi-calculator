
import { InputBoxManager } from './input_box.js';
import { NotificationManager } from './notifications.js';
import { MainButtonGrid } from './button_grid/main_button_grid.js';
import { KeyInput } from './keyboard.js';
import * as KeyDispatcher from './keyboard/dispatcher.js';
import { RightPanelManager } from './right_panel.js';
import * as Page from './page.js';
import { TAURI } from './tauri_api.js';
import { showPopup, PopupDisplayArgs, PopupDisplayHtml } from './popup_display.js';

import { OsType } from '@tauri-apps/plugin-os';

export class UiManager {
  readonly inputManager: InputBoxManager;
  readonly notificationManager: NotificationManager;
  readonly rightPanelManager: RightPanelManager;
  readonly osType: OsType;

  private keyHandler: KeyDispatcher.KeyEventHandler;
  private keyEventListener: (event: KeyboardEvent) => Promise<void>;

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
      modifierArgPanel: Page.getModifierArgPanel(),
      initialGrid: new MainButtonGrid(this.notificationManager),
      undoButton: Page.getUndoButton(),
      redoButton: Page.getRedoButton(),
      radiobuttonsDiv: Page.getTouchModesDiv(),
      valueStackDiv: Page.getValueStack(),
      uiManager: this,
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
    this.keyEventListener = (event) => this.dispatchOnKey(event);
  }

  static async create(): Promise<UiManager> {
    const osType = await TAURI.osType();
    return new UiManager(osType);
  }

  initListeners(): void {
    this.inputManager.initListeners();
    this.notificationManager.initListeners();
    this.rightPanelManager.initListeners();
    document.body.addEventListener("keydown", this.keyEventListener);
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

  showPopup(newHtml: PopupDisplayHtml, backButtonQuerySelector?: string): void {
    const args: PopupDisplayArgs = {
      newHtml,
      backButtonQuerySelector,
      onInit: () => {
        document.body.removeEventListener("keydown", this.keyEventListener);
      },
      onReturn: () => {
        document.body.addEventListener("keydown", this.keyEventListener);
      },
    };
    showPopup(args);
  }

}
