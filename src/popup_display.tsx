
import { ReactElement } from 'jsx-dom';

export const BACK_BUTTON_ID = "viewable-button-bar-back-button";
export const BACK_BUTTON_SELECTOR = `#${BACK_BUTTON_ID}`;

let POPUP_NESTING_COUNTER = 0;

// A type capable of showing popups.
export interface PopupManager {
  showPopup(newHtml: PopupDisplayHtml, backButtonQuerySelector?: string): void;
}

export function showPopup(args: PopupDisplayArgs): void {
  const oldHtml = [...document.body.children];
  if (args.newHtml instanceof DocumentFragment) {
    document.body.innerHTML = "";
    document.body.append(...args.newHtml.children);
  } else if (args.newHtml instanceof HTMLElement || args.newHtml instanceof SVGElement) {
    document.body.innerHTML = "";
    document.body.appendChild(args.newHtml);
  } else {
    document.body.innerHTML = args.newHtml;
  }

  POPUP_NESTING_COUNTER++;
  const nestingCounter = POPUP_NESTING_COUNTER;
  let keyListener: undefined | ((e: KeyboardEvent) => void) = undefined;
  const onReturn = () => {
    POPUP_NESTING_COUNTER--;
    args.onReturn();
    document.body.innerHTML = "";
    document.body.append(...oldHtml);
    if (keyListener != undefined) {
      document.body.removeEventListener("keydown", keyListener);
      keyListener = undefined;
    }
  };
  keyListener = (event) => {
    if ((event.key === "Escape") && (nestingCounter == POPUP_NESTING_COUNTER)) {
      onReturn();
    }
  };
  document.body.addEventListener("keydown", keyListener);

  if (args.backButtonQuerySelector) {
    initBackButton(args.backButtonQuerySelector, onReturn);
  }

  args.onInit();
}

function initBackButton(query: string, onReturn: () => void): void {
  const element = document.body.querySelector(query);
  if (element == undefined) {
    console.warn(`Query "${query}" not found.`);
    return;
  }
  element.addEventListener("click", onReturn);
}

export interface PopupDisplayArgs {
  newHtml: PopupDisplayHtml;
  backButtonQuerySelector?: string;

  onInit(): void;
  onReturn(): void;
}

export type PopupDisplayHtml = string | ReactElement;

export function generateViewPopupHtml(innerHTML: ReactElement): ReactElement {
  return <>
    <header>
      <div class="viewable-button-bar">
        <button id={BACK_BUTTON_ID}>Back</button>
      </div>
    </header>
    <main class="viewable-display-main">
      <span class="viewable-display-content-area">
        {innerHTML}
      </span>
    </main>
  </>;
}
