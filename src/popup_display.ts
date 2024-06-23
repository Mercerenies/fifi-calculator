
export function showPopup(args: PopupDisplayArgs): void {
  const oldHtml = document.body.innerHTML;
  document.body.innerHTML = args.newHtml;

  if (args.backButtonQuerySelector) {
    initBackButton(args.backButtonQuerySelector, oldHtml, args);
  }

  args.onInit();
}

function initBackButton(query: string, oldHtml: string, args: PopupDisplayArgs): void {
  const element = document.body.querySelector(query);
  if (element == undefined) {
    console.warn(`Query "${query}" not found.`);
    return;
  }
  element.addEventListener("click", () => {
    args.onReturn();
    document.body.innerHTML = oldHtml;
  });
}

export interface PopupDisplayArgs {
  newHtml: string;
  backButtonQuerySelector?: string;

  onInit(): void;
  onReturn(): void;
}
