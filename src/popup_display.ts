
export function showPopup(args: PopupDisplayArgs): void {
  const oldHtml = [...document.body.children];
  if (args.newHtml instanceof HTMLElement) {
    document.body.innerHTML = "";
    document.body.appendChild(args.newHtml);
  } else {
    document.body.innerHTML = args.newHtml;
  }

  let keyListener: undefined | ((e: KeyboardEvent) => void) = undefined;
  const onReturn = () => {
    args.onReturn();
    document.body.innerHTML = "";
    document.body.append(...oldHtml);
    if (keyListener != undefined) {
      document.body.removeEventListener("keydown", keyListener);
      keyListener = undefined;
    }
  };
  keyListener = (event) => {
    if (event.key === "Escape") {
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
  newHtml: string | HTMLElement;
  backButtonQuerySelector?: string;

  onInit(): void;
  onReturn(): void;
}
