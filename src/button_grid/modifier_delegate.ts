
import { KeyEventInput, KeyResponse } from '../keyboard.js';
import * as Dispatcher from '../keyboard/dispatcher.js';

// A ModifierDelegate can preempt key events from the button grid and
// respond to them. It also responds to resetModifiers().
export interface ModifierDelegate {
  getModifiers(): ButtonModifiers;
  resetModifiers(): void;

  onKeyDown(input: KeyEventInput): Promise<KeyResponse>;
}

export const NULL_DELEGATE: ModifierDelegate = {
  getModifiers() {
    return defaultModifiers();
  },
  resetModifiers() {
    // No action; NULL_DELEGATE does not store any state.
  },
  onKeyDown() {
    return Promise.resolve(KeyResponse.PASS);
  },
};

export function delegates(delegates: readonly ModifierDelegate[]): ModifierDelegate {
  const keyHandler = Dispatcher.sequential(delegates);
  return {
    getModifiers() {
      return delegates.map((d) => d.getModifiers()).reduce(appendModifiers, defaultModifiers());
    },
    resetModifiers() {
      delegates.forEach((d) => d.resetModifiers());
    },
    onKeyDown(input) {
      return keyHandler.onKeyDown(input);
    },
  };
}

// ButtonModifiers acts as a monoid with defaultModifiers() as the
// identity and appendModifiers as the monoid operation.
export interface ButtonModifiers {
  prefixArgument: number | undefined;
  keepModifier: boolean;
}

export function modifiersToRustArgs(modifiers: ButtonModifiers): CommandOptions {
  return {
    argument: modifiers.prefixArgument ?? null,
    keepModifier: modifiers.keepModifier,
  };
}

export function defaultCommandOptions(): CommandOptions {
  return modifiersToRustArgs(defaultModifiers());
}

export function defaultModifiers(): ButtonModifiers {
  return {
    prefixArgument: undefined,
    keepModifier: false,
  };
}

export function appendModifiers(left: ButtonModifiers, right: ButtonModifiers): ButtonModifiers {
  // Boolean modifiers are combined using ||, the prefixArgument is
  // combined using the 'First' monoid.
  return {
    prefixArgument: left.prefixArgument ?? right.prefixArgument,
    keepModifier: left.keepModifier || right.keepModifier,
  };
}
