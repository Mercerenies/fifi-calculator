
import * as Page from './page.js';
import { InputBoxManager } from './input_box.js';
import { NotificationManager } from './notifications.js';
import { MainButtonGrid } from './button_grid/main_button_grid.js';
import { KeyInput, KeyResponse } from './keyboard.js';
import { RightPanelManager } from './right_panel.js';

const { listen } = window.__TAURI__.event;
const os = window.__TAURI__.os;

class UiManager {
  readonly inputManager: InputBoxManager;
  readonly notificationManager: NotificationManager;
  readonly rightPanelManager: RightPanelManager;
  readonly osType: OsType;

  constructor(osType: OsType) {
    this.inputManager = new InputBoxManager({
      inputBox: Page.getInputBoxDiv(),
      inputTextBox: Page.getInputTextBox(),
      inputLabel: Page.getInputTextBoxLabel(),
    });
    this.notificationManager = new NotificationManager(Page.getNotificationBox());
    this.rightPanelManager = new RightPanelManager(
      Page.getButtonGridContainer(),
      new MainButtonGrid(this.inputManager, this.notificationManager),
    );
    this.osType = osType;
  }

  static async create(): Promise<UiManager> {
    const osType = await os.type();
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

    if (document.activeElement === Page.getInputTextBox()) {
      const keyResponse = await this.inputManager.onKeyDown(input);
      if (keyResponse === KeyResponse.BLOCK) {
        return;
      }
    }
    await this.rightPanelManager.onKeyDown(input);
  }
}

function refreshStack(newStack: string[]): void {
  const ol = document.createElement("ol");
  for (let i = 0; i < newStack.length; i++) {
    const elem = newStack[i];
    const li = document.createElement("li");
    li.value = newStack.length - i;
    li.innerHTML = elem;
    ol.appendChild(li);
  }
  const stack = Page.getValueStack();
  stack.innerHTML = "";
  stack.appendChild(ol);
}

window.addEventListener("DOMContentLoaded", async function() {
  const uiManager = await UiManager.create();
  uiManager.initListeners();
  await listen("refresh-stack", (event) => refreshStack(event.payload.stack));
  await listen("show-error", (event) => uiManager.notificationManager.show(event.payload.errorMessage));
});
