
import * as Page from './page.js';
import { InputBoxManager, KeyResponse } from './input_box.js';
import { NotificationManager } from './notifications.js';
import { ButtonGridManager } from './button_grid.js';
import { MainButtonGrid } from './button_grid/main_button_grid.js';
import { KeyInput } from './keyboard.js';

const { listen } = window.__TAURI__.event;
const os = window.__TAURI__.os;

class UiManager {
  readonly inputManager: InputBoxManager;
  readonly notificationManager: NotificationManager;
  readonly buttonGridManager: ButtonGridManager;
  readonly osType: OsType;

  constructor(osType: OsType) {
    this.inputManager = new InputBoxManager({
      inputBox: Page.getInputBoxDiv(),
      inputTextBox: Page.getInputTextBox(),
      inputLabel: Page.getInputTextBoxLabel(),
    });
    this.notificationManager = new NotificationManager(Page.getNotificationBox());
    this.buttonGridManager = new ButtonGridManager(
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
    await this.buttonGridManager.onKeyDown(input);
  }
}

function refreshStack(newStack: string[]): void {
  const ol = document.createElement("ol");
  for (let i = newStack.length - 1; i >= 0; i--) {
    const elem = newStack[i];
    const li = document.createElement("li");
    li.value = i + 1;
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
