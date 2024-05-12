
import * as Page from './page.js';
import { InputBoxManager, NumericalInputMethod, KeyResponse } from './input_box.js';
import { NotificationManager } from './notifications.js';
import { ButtonGridManager, MainButtonGrid } from './button_grid.js';

const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;

class UiManager {
  readonly inputManager: InputBoxManager;
  readonly notificationManager: NotificationManager;
  readonly buttonGridManager: ButtonGridManager

  constructor() {
    this.inputManager = new InputBoxManager({
      inputBox: Page.getInputBoxDiv(),
      inputTextBox: Page.getInputTextBox(),
      inputLabel: Page.getInputTextBoxLabel(),
    });
    this.notificationManager = new NotificationManager(Page.getNotificationBox());
    this.buttonGridManager = new ButtonGridManager(new MainButtonGrid(this.inputManager));
  }

  initListeners(): void {
    this.inputManager.initListeners();
    this.notificationManager.initListeners();
    document.body.addEventListener("keydown", (event) => this.dispatchOnKey(event));
  }

  private async dispatchOnKey(event: KeyboardEvent): Promise<void> {
    if (document.activeElement === Page.getInputTextBox()) {
      const keyResponse = await this.inputManager.onKeyDown(event);
      if (keyResponse === KeyResponse.BLOCK) {
        return;
      }
    }
    await this.buttonGridManager.onKeyDown(event);
  }
}

function refreshStack(newStack: string[]): void {
  const ol = document.createElement("ol");
  for (let i = newStack.length - 1; i >= 0; i--) {
    const elem = newStack[i];
    const li = document.createElement("li");
    li.value = i;
    li.innerHTML = elem;
    ol.appendChild(li);
  }
  const stack = Page.getValueStack();
  stack.innerHTML = "";
  stack.appendChild(ol);
}

window.addEventListener("DOMContentLoaded", async function() {
  const uiManager = new UiManager();
  uiManager.initListeners();
  await listen("refresh-stack", (event) => refreshStack(event.payload.stack));
  await listen("show-error", (event) => uiManager.notificationManager.show(event.payload.errorMessage));
});
