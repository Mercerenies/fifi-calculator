
import { HelpModeButtonManager, HelpModeResponse } from './button_grid/help_text.js';
import { AbstractButtonManager, GridCell } from './button_grid.js';
import { PopupManager, generateViewPopupHtml, BACK_BUTTON_SELECTOR } from './popup_display.js';
import { jsx, Fragment } from './jsx.js';

export class HelpManager {
  private helpButton: HTMLButtonElement;
  private buttonGridManager: AbstractButtonManager;
  private popupManager: PopupManager;

  constructor(args: HelpManagerConstructorArgs) {
    this.helpButton = args.helpButton;
    this.buttonGridManager = args.buttonGridManager;
    this.popupManager = args.popupManager;
  }

  initListeners(): void {
    this.helpButton.addEventListener("click", () => this.onHelpButtonClicked());
  }

  private onHelpButtonClicked(): void {
    if (this.buttonGridManager.getCurrentManager() instanceof HelpModeButtonManager) {
      this.buttonGridManager.getCurrentManager().onEscape(); // Fire and forget; simulate a cancel action.
    } else {
      const helpMgr = new HelpModeButtonManager(this.buttonGridManager, (cell) => this.showHelpFor(cell));
      this.buttonGridManager.setCurrentManager(helpMgr);
    }
  }

  private showHelpFor(cell: GridCell): Promise<HelpModeResponse> {
    const helpHtml = cell.getHelpPage();
    if (helpHtml === undefined) {
      // Nothing to show
      return Promise.resolve("pass");
    }
    this.popupManager.showPopup(generateViewPopupHtml(helpPageToJsx(cell, helpHtml)), BACK_BUTTON_SELECTOR);
    return Promise.resolve("success");
  }
}

export interface HelpManagerConstructorArgs {
  readonly helpButton: HTMLButtonElement;
  readonly buttonGridManager: AbstractButtonManager;
  readonly popupManager: PopupManager;
}

export interface HelpPage {
  readonly headerText: string;
  readonly body: JSX.Element;
}

export function helpPageToJsx(cell: GridCell, page: HelpPage): JSX.Element {
  return <>
    <h1>{page.headerText}</h1>
    <div>{page.body}</div>
  </>;
}
