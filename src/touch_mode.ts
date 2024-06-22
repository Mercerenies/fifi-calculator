
import { StackUpdatedDelegate } from './stack_view.js';

// Manager for the "Touch Mode" radiobuttons which control what
// happens when you click/drag a stack element.
export class TouchModeManager implements StackUpdatedDelegate {
  private radiobuttonsDiv: HTMLElement;
  private valueStackDiv: HTMLElement;
  private touchMode: TouchMode = TouchMode.DRAG;

  constructor(args: TouchModeManagerArgs) {
    this.radiobuttonsDiv = args.radiobuttonsDiv;
    this.valueStackDiv = args.valueStackDiv;
  }

  initListeners(): void {
    for (const inputElem of this.radiobuttonsDiv.querySelectorAll('input[type="radio"]')) {
      const touchModeData = (inputElem as HTMLElement).dataset.touchMode;
      if (touchModeData == undefined) {
        throw "No touchMode data on radio button";
      }
      const touchMode = TouchMode[touchModeData as keyof typeof TouchMode];
      if (touchMode == undefined) {
        throw "Invalid touchMode data on radio button";
      }
      inputElem.addEventListener('change', () => {
        this.touchMode = touchMode;
        this.updateTouchMode();
      });
    }
  }

  private updateTouchMode(): void {
    console.log(TouchMode[this.touchMode]);
  }

  async onStackUpdated(): Promise<void> {
    this.updateTouchMode();
  }
}

export interface TouchModeManagerArgs {
  radiobuttonsDiv: HTMLElement;
  valueStackDiv: HTMLElement;
}

export enum TouchMode {
  DRAG,
  VIEW,
  EDIT,
}
