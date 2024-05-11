
import { getInputBoxDiv, getInputTextBox, getValueStack } from './page.js';

const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;

const NUMBERS = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"];

const SIMPLE_DISPATCH_KEYS: Record<string, string> = {
  "+": "+",
  "-": "-",
  "*": "*",
  "/": "/",
};

function showInputBox(): void {
  const inputBox = getInputBoxDiv();
  inputBox.style.display = "flex";
  window.setTimeout(() => getInputTextBox().focus(), 1);
}

function hideInputBox(): void {
  const inputBox = getInputBoxDiv();
  const inputTextBox = getInputTextBox();
  inputBox.style.display = "none";
  inputTextBox.value = "";
}

async function submitInteger() {
  const inputTextBox = getInputTextBox();
  const text = inputTextBox.value;
  await invoke('submit_integer', { value: +text });
  hideInputBox();
}

async function dispatchOnKeyInputField(event: KeyboardEvent): Promise<void> {
  if (event.key === "Escape") {
    // Remove focus and cancel, if the key is ESCAPE
    hideInputBox();
    event.preventDefault();
  } else if (event.key === "Enter") {
    // Submit if key is ENTER
    await submitInteger();
    event.preventDefault();
  } else if (event.key in SIMPLE_DISPATCH_KEYS) {
    // If the key is a dispatch key, submit and then dispatch.
    await submitInteger();
    dispatchOnKeyGeneral(event);
  }
}

async function dispatchOnKeyGeneral(event: KeyboardEvent): Promise<void> {
  if (NUMBERS.includes(event.key)) {
    const inputTextBox = getInputTextBox();
    showInputBox();
    inputTextBox.value = event.key;
    event.preventDefault();
  } else if (event.key in SIMPLE_DISPATCH_KEYS) {
    invoke('math_command', { commandName: SIMPLE_DISPATCH_KEYS[event.key] });
    event.preventDefault();
  }
}

function dispatchOnKey(event: KeyboardEvent): Promise<void> {
  if (document.activeElement === getInputTextBox()) {
    return dispatchOnKeyInputField(event);
  } else {
    return dispatchOnKeyGeneral(event);
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
  const stack = getValueStack();
  stack.innerHTML = "";
  stack.appendChild(ol);
}

window.addEventListener("DOMContentLoaded", async function() {
  const inputTextbox = getInputTextBox();

  document.body.addEventListener("keydown", dispatchOnKey);
  inputTextbox.addEventListener("focusout", () => hideInputBox());
  await listen("refresh-stack", (event) => refreshStack(event.payload.stack));
});
