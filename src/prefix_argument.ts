
// Handler for the Emacs-style prefix argument.
//
// This subsystem functions as a state machine with three states: the
// default state, the C-u multiplier state, and the numerical input
// state. Generally, the numerical input state will take precedent
// over the multiplier state.
//
// Additionally, we have the negative state, which is basically the
// numerical input state except that it treats its next input as
// negative.

import { KeyResponse } from './keyboard.js';
import { Signal } from './signal.js';

export type SimpleInput = "C-u" | "-" | "C--" | "Escape";
export type ArgInput = "C-#" | "#";

export type StateTransition =
  { input: SimpleInput } | { input: ArgInput, argument: number };

export interface PrefixArgState {
  readonly prefixArgument: number | null;

  // Returns null if the state shouldn't change and we shouldn't
  // absorb the input for the state machine.
  onTransition(transition: StateTransition): PrefixArgState | null;
}

export class PrefixArgStateMachine {
  private _currentState: PrefixArgState;
  readonly stateChangedSignal: Signal<StateChangedEvent> = new Signal<StateChangedEvent>();

  constructor(initialState: PrefixArgState = DEFAULT_STATE) {
    this._currentState = initialState;
  }

  get prefixArgument(): number | null {
    return this.currentState.prefixArgument;
  }

  get currentState(): PrefixArgState {
    return this._currentState;
  }

  set currentState(newState: PrefixArgState) {
    const oldState = this._currentState;
    this._currentState = newState;
    this.stateChangedSignal.emit({
      type: "prefix-arg-state-changed",
      oldState,
      newState,
    });
  }

  onTransition(transition: StateTransition): KeyResponse {
    const newState = this.currentState.onTransition(transition);
    if (newState === null) {
      return KeyResponse.PASS;
    } else {
      this.currentState = newState;
      return KeyResponse.BLOCK;
    }
  }
}

export interface StateChangedEvent {
  type: "prefix-arg-state-changed";
  oldState: PrefixArgState;
  newState: PrefixArgState;
}

export const DEFAULT_STATE: PrefixArgState = {
  prefixArgument: null,
  onTransition(transition: StateTransition): PrefixArgState | null {
    switch (transition.input) {
    case "C-u":
      return multiplierState(1);
    case "C-#":
      return numericalState(transition.argument);
    case "C--":
      return NEGATIVE_INPUT_STATE;
    case "Escape":
    case "#":
    case "-":
      return null; // No action
    }
  },
};

export function multiplierState(k: number): PrefixArgState {
  return {
    prefixArgument: 4 * k,
    onTransition(transition: StateTransition): PrefixArgState | null {
      switch (transition.input) {
      case "C-u":
        return multiplierState(k + 1);
      case "C-#":
      case "#":
        return numericalState(transition.argument);
      case "-":
      case "C--":
        return NEGATIVE_INPUT_STATE;
      case "Escape":
        return DEFAULT_STATE;
      }
    },
  }
}

export const NEGATIVE_INPUT_STATE: PrefixArgState = {
  prefixArgument: -1,
  onTransition(transition: StateTransition): PrefixArgState | null {
    switch (transition.input) {
    case "C-u":
    case "-":
    case "C--":
      return null; // Ignore the input; it does nothing at this point.
    case "C-#":
    case "#":
      return numericalState(- transition.argument);
    case "Escape":
        return DEFAULT_STATE;
    }
  }
}

export function numericalState(n: number): PrefixArgState {
  return {
    prefixArgument: n,
    onTransition(transition: StateTransition): PrefixArgState | null {
      switch (transition.input) {
      case "C-u":
      case "-":
      case "C--":
        return null; // Ignore the input; it does nothing at this point.
      case "C-#":
      case "#":
        return numericalState(10 * n + transition.argument);
      case "Escape":
        return DEFAULT_STATE;
      }
    },
  }
}
