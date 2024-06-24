
import { UiManager } from '../ui_manager.js';
import { TouchModeFactoryContext } from '../touch_mode.js';
import { ClickableTouchMode } from './clickable.js';
import { jsx, HtmlText, Fragment } from '../jsx.js';

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
    const htmlToDisplay = contentElem.innerHTML;
    this.uiManager.showPopup(this.getHtml(htmlToDisplay), "#viewable-button-bar-back-button");
  }

  private getHtml(innerHTML: string): Fragment {
    return <>
      <header>
        <div class="viewable-button-bar">
          <button id="viewable-button-bar-back-button">Back</button>
        </div>
      </header>
      <main class="viewable-display-main">
        <span class="viewable-display-content-area">
          <HtmlText content={innerHTML} />
        </span>
      </main>
    </>;
  }
}
