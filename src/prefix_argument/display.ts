
// Display and on-screen buttons for managing the prefix argument.

import { PrefixArgStateMachine, StateTransition, DEFAULT_STATE } from '../prefix_argument.js';

export class PrefixArgumentDisplay {
  private panelNode: HTMLElement;
  private stateMachine: PrefixArgStateMachine;

  constructor(panelNode: HTMLElement, stateMachine: PrefixArgStateMachine) {
    this.panelNode = panelNode;
    this.stateMachine = stateMachine;
  }

  initListeners(): void {
    
  }

}
