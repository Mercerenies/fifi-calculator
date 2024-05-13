
import { KeyEventInput, KeyResponse } from '../keyboard.js';
import { ModifierDelegate, ButtonModifiers } from '../button_grid/modifier_delegate.js';
import { PrefixArgStateMachine, StateTransition, DEFAULT_STATE } from '../prefix_argument.js';

// Modifier delegate that handles prefix arguments.
export class PrefixArgumentDelegate implements ModifierDelegate {
  private stateMachine: PrefixArgStateMachine;

  constructor(stateMachine: PrefixArgStateMachine) {
    this.stateMachine = stateMachine;
  }

  getModifiers(): ButtonModifiers {
    return {
      prefixArgument: this.stateMachine.prefixArgument,
    };
  }

  resetModifiers(): void {
    this.stateMachine.currentState = DEFAULT_STATE;
  }

  onKeyDown(input: KeyEventInput): Promise<KeyResponse> {
    const transition = keyToStateTransition(input.toEmacsSyntax());
    if (transition !== null) {
      return Promise.resolve(this.stateMachine.onTransition(transition));
    } else {
      return Promise.resolve(KeyResponse.PASS);
    }
  }
}

// This one is just a massive switch statement, converting all of the
// keys we care about into state transitions. *Maybe* we could shorten
// it a bit, but it's honestly probably not worth it. One giant switch
// statement isn't that bad.
function keyToStateTransition(key: string): StateTransition | null {
  switch (key) {
  case "C-u":
  case "-":
  case "C--":
  case "Escape":
    return { input: key }
  case "0":
  case "1":
  case "2":
  case "3":
  case "4":
  case "5":
  case "6":
  case "7":
  case "8":
  case "9":
    return { input: "#", argument: +key };
  case "C-0":
  case "M-0":
  case "C-M-0":
    return { input: "C-#", argument: 0 };
  case "C-1":
  case "M-1":
  case "C-M-1":
    return { input: "C-#", argument: 1 };
  case "C-2":
  case "M-2":
  case "C-M-2":
    return { input: "C-#", argument: 2 };
  case "C-3":
  case "M-3":
  case "C-M-3":
    return { input: "C-#", argument: 3 };
  case "C-4":
  case "M-4":
  case "C-M-4":
    return { input: "C-#", argument: 4 };
  case "C-5":
  case "M-5":
  case "C-M-5":
    return { input: "C-#", argument: 5 };
  case "C-6":
  case "M-6":
  case "C-M-6":
    return { input: "C-#", argument: 6 };
  case "C-7":
  case "M-7":
  case "C-M-7":
    return { input: "C-#", argument: 7 };
  case "C-8":
  case "M-8":
  case "C-M-8":
    return { input: "C-#", argument: 8 };
  case "C-9":
  case "M-9":
  case "C-M-9":
    return { input: "C-#", argument: 9 };
  default:
    return null;
  }
}
