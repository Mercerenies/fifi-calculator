
const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;

const NUMBERS = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"];

const ElementIds = {
  INPUT_BOX: 'input-box',
  INPUT_TEXTBOX: 'input-textbox',
  VALUE_STACK: 'value-stack',
};

function getElement(id: string): HTMLElement {
  const element = document.getElementById(id);
  if (element === null) {
    throw `No element with ID $id`;
  } else {
    return element;
  }
}

function showInputBox(): void {
  const inputBox = getElement(ElementIds.INPUT_BOX);
  inputBox.style.display = "block";
  window.setTimeout(() => getElement(ElementIds.INPUT_TEXTBOX).focus(), 1);
}

function hideInputBox(): void {
  const inputBox = getElement(ElementIds.INPUT_BOX);
  const inputTextBox = getElement(ElementIds.INPUT_TEXTBOX) as HTMLInputElement;
  inputBox.style.display = "none";
  inputTextBox.value = "";
}

async function dispatchOnKeyInputField(event: KeyboardEvent): Promise<void> {
  // Remove focus if the key is ESCAPE
  if (event.key === "Escape") {
    hideInputBox();
    event.preventDefault();
  }

  // Submit if key is ENTER
  if (event.key === "Enter") {
    const inputTextBox = getElement(ElementIds.INPUT_TEXTBOX) as HTMLInputElement;
    const text = inputTextBox.value;
    await invoke('submit_integer', { value: +text });
    hideInputBox();
    event.preventDefault();
  }
}

async function dispatchOnKeyGeneral(event: KeyboardEvent): Promise<void> {
  if (NUMBERS.includes(event.key)) {
    const inputTextBox = getElement(ElementIds.INPUT_TEXTBOX) as HTMLInputElement;
    showInputBox();
    inputTextBox.value = event.key;
    event.preventDefault();
  }
}

function dispatchOnKey(event: KeyboardEvent): Promise<void> {
  if (document.activeElement === getElement(ElementIds.INPUT_TEXTBOX)) {
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
  const stack = getElement(ElementIds.VALUE_STACK);
  stack.innerHTML = "";
  stack.appendChild(ol);
}

window.addEventListener("DOMContentLoaded", async function() {
  const inputTextbox = getElement(ElementIds.INPUT_TEXTBOX);

  document.body.addEventListener("keydown", dispatchOnKey);
  inputTextbox.addEventListener("focusout", () => hideInputBox());
  await listen("refresh-stack", (event) => refreshStack(event.payload.stack));
});
