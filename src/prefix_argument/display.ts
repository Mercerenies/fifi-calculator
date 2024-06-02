
// Display and on-screen buttons for managing the prefix argument.

import { PrefixArgStateMachine, StateChangedEvent } from '../prefix_argument.js';
import { SignalListener } from '../signal.js';

export class PrefixArgumentDisplay {
  private panelNode: HTMLElement;
  private stateMachine: PrefixArgStateMachine;
  private argDisplayBox: HTMLInputElement;

  private signalListener: SignalListener<StateChangedEvent> | undefined;

  constructor(
    panelNode: HTMLElement,
    stateMachine: PrefixArgStateMachine,
    opts: Partial<PrefixArgumentDisplayOptions> = {},
  ) {
    this.panelNode = panelNode;
    this.stateMachine = stateMachine;
    // Load display box.
    const id = opts.displayBoxId ?? "prefix-arg-display-box";
    const element = panelNode.querySelector(`#${id}`);
    if (element && element instanceof HTMLInputElement) {
      this.argDisplayBox = element;
    } else {
      throw "Failed to find display box element with id " + id + ", got " + element;
    }
  }

  initListeners(): void {
    this.signalListener = (event) => this.onStateChanged(event);
    this.stateMachine.stateChangedSignal.addListener(this.signalListener);
    for (const button of this.panelNode.querySelectorAll("button")) {
      const buttonValue = button.dataset.prefixArg;
      if (buttonValue === undefined) {
        throw "Button doesn't have dataset.prefixArg attribute";
      }
      button.addEventListener("click", () => {
        this.stateMachine.onTransition({
          input: "displayed-button",
          argument: +buttonValue,
        });
      });
    }
  }

  uninitListeners(): void {
    if (this.signalListener) {
      this.stateMachine.stateChangedSignal.removeListener(this.signalListener);
      this.signalListener = undefined;
    }
  }

  private onStateChanged(event: StateChangedEvent): void {
    this.argDisplayBox.value = stringifyArg(event.newState.prefixArgument);
  }

}

export interface PrefixArgumentDisplayOptions {
  displayBoxId: string;
}

function stringifyArg(argument: number | null): string {
  if (argument === null) {
    return "";
  } else {
    return "" + argument;
  }
}
