
import { UiManager } from '../ui_manager.js';
import { TouchModeFactoryContext } from '../touch_mode.js';
import { ClickableTouchMode } from './clickable.js';
import { jsx, HtmlText, Fragment } from '../jsx.js';
import { DirectRenderTarget, getGraphicsElements, getGraphicsPayload, renderPlotTo } from '../graphics.js';

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
    const htmlToDisplay = getHtmlToDisplay(contentElem as HTMLElement);
    this.uiManager.showPopup(generateViewPageHtml(htmlToDisplay), `#${BACK_BUTTON_ID}`);
  }
}

const BACK_BUTTON_ID = "viewable-button-bar-back-button";

function isSoleGraphicsElement(contentElem: HTMLElement): boolean {
  if (contentElem.children.length !== 1) {
    return false;
  }
  const child = contentElem.children[0] as HTMLElement;
  return (getGraphicsPayload(child) !== undefined);
}

function getHtmlToDisplay(contentElem: HTMLElement): JSX.Element {
  if (isSoleGraphicsElement(contentElem)) {
    // Show interactive graph
    const graphicsElements = getGraphicsElements(contentElem);
    if (graphicsElements.length !== 1) {
      throw "Expected only 1 graphics element in view mode";
    }
    const payload = getGraphicsPayload(graphicsElements[0]);
    if (payload == undefined) {
      throw "Graphics element is missing data-graphics-payload attr";
    }
    const div = document.createElement("div");
    div.className = "plotly-interactive-plot";
    const renderTarget = new DirectRenderTarget(div);
    renderPlotTo(payload, renderTarget, {}, { responsive: true });
    return div;
  } else {
    // Show as plaintext
    return <HtmlText content={contentElem.innerHTML} />;
  }
}

function generateViewPageHtml(innerHTML: JSX.Element): JSX.Element {
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
