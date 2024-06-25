
import { Fragment, isFragment } from './jsx.js';

let POPUP_NESTING_COUNTER = 0;

export function showPopup(args: PopupDisplayArgs): void {
  const oldHtml = [...document.body.children];
  if (args.newHtml instanceof HTMLElement) {
    document.body.innerHTML = "";
    document.body.appendChild(args.newHtml);
  } else if (isFragment(args.newHtml)) {
    document.body.innerHTML = "";
    document.body.append(...args.newHtml.elements);
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

export type PopupDisplayHtml = string | HTMLElement | Fragment;
