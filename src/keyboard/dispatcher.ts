
// Keyboard key event dispatcher.

import { KeyEventInput, KeyResponse } from '../keyboard.js';

export interface KeyEventHandler {
  onKeyDown(input: KeyEventInput): Promise<KeyResponse>;
}

export function sequential(handlers: readonly KeyEventHandler[]): KeyEventHandler {
  return {
    async onKeyDown(input: KeyEventInput): Promise<KeyResponse> {
      for (const handler of handlers) {
        const response = await handler.onKeyDown(input);
        if (response === KeyResponse.BLOCK) {
          return response;
        }
      }
      return KeyResponse.PASS;
    }
  };
}

export function filtered(handler: KeyEventHandler, condition: (input: KeyEventInput) => boolean): KeyEventHandler {
  return {
    onKeyDown(input: KeyEventInput): Promise<KeyResponse> {
      if (condition(input)) {
        return handler.onKeyDown(input);
      } else {
        return Promise.resolve(KeyResponse.PASS);
      }
    }
  };
}
