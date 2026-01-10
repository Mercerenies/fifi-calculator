
import { UiManager } from '../ui_manager.js';
import { TouchModeFactoryContext } from '../touch_mode.js';
import { ClickableTouchMode } from './clickable.js';
import { DirectRenderTarget, getGraphicsElements, getGraphicsPayload, renderPlotTo } from '../graphics.js';
import { generateViewPopupHtml, BACK_BUTTON_ID, BACK_BUTTON_SELECTOR } from '../popup_display.js';

import { ReactElement } from 'jsx-dom';

export class ViewTouchMode extends ClickableTouchMode {
  private uiManager: UiManager;

  constructor(context: TouchModeFactoryContext) {
    super(context);
    this.uiManager = context.uiManager;
  }

  onClick(elem: HTMLElement): void {
    const contentElem = elem.getElementsByClassName("value-stack-element-value")[0];
    if (!contentElem) {
      console.warn("Could not find content element for view mode");
      return;
    }
    const htmlToDisplay = getHtmlToDisplay(contentElem as HTMLElement, this.uiManager);
    this.uiManager.showPopup(generateViewPopupHtml(htmlToDisplay), BACK_BUTTON_SELECTOR);
  }
}

function isSoleGraphicsElement(contentElem: HTMLElement): boolean {
  if (contentElem.children.length !== 1) {
    return false;
  }
  const child = contentElem.children[0] as HTMLElement;
  return (getGraphicsPayload(child) !== undefined);
}

function getHtmlToDisplay(contentElem: HTMLElement, uiManager: UiManager): HTMLElement {
  if (getGraphicsPayload(contentElem) != undefined) {
    return showInteractiveGraph(contentElem);
  } else if (isSoleGraphicsElement(contentElem)) {
    // Show interactive graph
    const graphicsElements = getGraphicsElements(contentElem);
    if (graphicsElements.length !== 1) {
      throw "Expected only 1 graphics element in view mode";
    }
    return showInteractiveGraph(graphicsElements[0] as HTMLElement);
  } else {
    // Show as plaintext (make any graphs clickable)
    const newSpan = document.createElement('span');
    newSpan.innerHTML = contentElem.innerHTML;
    for (const graphicsElement of getGraphicsElements(newSpan)) {
      const button = insertButton(graphicsElement as HTMLElement);
      button.addEventListener('click', () => {
        const htmlToDisplay = getHtmlToDisplay(graphicsElement as HTMLElement, uiManager);
        uiManager.showPopup(generateViewPopupHtml(htmlToDisplay), BACK_BUTTON_SELECTOR);
      });
    }
    return newSpan;
  }
}

function showInteractiveGraph(graphicsElement: HTMLElement): HTMLElement {
  const payload = getGraphicsPayload(graphicsElement);
  if (payload == undefined) {
    throw "Graphics element is missing data-graphics-payload attr";
  }
  const div = document.createElement("div");
  div.className = "plotly-interactive-plot";
  const renderTarget = new DirectRenderTarget(div);
  renderPlotTo(payload, renderTarget, {}, { responsive: true });
  return div;
}

function insertButton(element: HTMLElement): HTMLButtonElement {
  const parent = element.parentNode;
  if (parent == null) {
    throw "Could not find parent for HTMLElement";
  }
  const elementIndex = Array.prototype.indexOf.call(parent.childNodes, element);
  parent.removeChild(element);
  const button = document.createElement("button");
  button.appendChild(element);
  parent.insertBefore(button, parent.childNodes[elementIndex] ?? null);
  return button;
}

function generateViewPageHtml(innerHTML: ReactElement): ReactElement {
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
