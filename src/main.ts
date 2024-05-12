
import * as Page from './page.js';
import { InputBoxManager, NumericalInputMethod, KeyResponse } from './input_box.js';
import { NotificationManager } from './notifications.js';

const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;

const NUMBERS = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"];

const SIMPLE_DISPATCH_KEYS: Record<string, string> = {
  "+": "+",
  "-": "-",
  "*": "*",
  "/": "/",
};

class UiManager {
  readonly inputManager: InputBoxManager;
  readonly notificationManager: NotificationManager;

  constructor() {
    this.inputManager = new InputBoxManager({
      inputBox: Page.getInputBoxDiv(),
      inputTextBox: Page.getInputTextBox(),
      inputLabel: Page.getInputTextBoxLabel(),
    });
    this.notificationManager = new NotificationManager(Page.getNotificationBox());
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
    await this.dispatchOnKeyGeneral(event);
  }

  private async dispatchOnKeyGeneral(event: KeyboardEvent): Promise<void> {
    if (NUMBERS.includes(event.key)) {
      event.preventDefault();
      this.inputManager.show(new NumericalInputMethod());
      this.inputManager.setTextBoxValue(event.key);
    } else if (event.key in SIMPLE_DISPATCH_KEYS) {
      event.preventDefault();
      await invoke('math_command', { commandName: SIMPLE_DISPATCH_KEYS[event.key] });
    }
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
