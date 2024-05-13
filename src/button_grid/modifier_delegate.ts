
import { KeyEventInput, KeyResponse } from '../keyboard.js';

// A ModifierDelegate can preempt key events from the button grid and
// respond to them. It also responds to resetModifiers().
export interface ModifierDelegate {
  getModifiers(): ButtonModifiers;
  resetModifiers(): void;

  onKeyDown(input: KeyEventInput): Promise<KeyResponse>;
}

export const NULL_DELEGATE: ModifierDelegate = {
  getModifiers() {
    return { prefixArgument: null };
  },
  resetModifiers() {
    // No action; NULL_DELEGATE does not store any state.
  },
  onKeyDown() {
    return Promise.resolve(KeyResponse.PASS);
  },
};

export interface ButtonModifiers {
  prefixArgument: number | null;
}
