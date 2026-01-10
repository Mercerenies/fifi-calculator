
import { HelpModeButtonManager, HelpModeResponse } from './button_grid/help_text.js';
import { ButtonGridManager, GridCell } from './button_grid.js';
import { PopupManager, generateViewPopupHtml, BACK_BUTTON_SELECTOR } from './popup_display.js';
import { KeyEventHandler } from './keyboard/dispatcher.js';
import { KeyEventInput, KeyResponse } from './keyboard.js';

import { ReactElement } from "jsx-dom";

export class HelpManager implements KeyEventHandler {
  private helpButton: HTMLButtonElement;
  private buttonGridManager: ButtonGridManager;
  private popupManager: PopupManager;

  constructor(args: HelpManagerConstructorArgs) {
    this.helpButton = args.helpButton;
    this.buttonGridManager = args.buttonGridManager;
    this.popupManager = args.popupManager;
  }

  initListeners(): void {
    this.helpButton.addEventListener("click", () => this.onHelpButtonClicked());
  }

  onKeyDown(input: KeyEventInput): Promise<KeyResponse> {
    console.log(input.key);
    if (input.key != "?") {
      return Promise.resolve(KeyResponse.PASS);
    }
    this.onHelpButtonClicked();
    return Promise.resolve(KeyResponse.BLOCK);
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
  readonly buttonGridManager: ButtonGridManager;
  readonly popupManager: PopupManager;
}

export interface HelpPage {
  readonly headerText: string;
  readonly body: ReactElement;
}

export function helpPageToJsx(cell: GridCell, page: HelpPage): ReactElement {
  const cellHtml = cell.getInnerHTML();
  return <>
    <h1><span class="helptext-icon">{cellHtml}</span> {page.headerText}</h1>
    <div>{page.body}</div>
  </>;
}
