
// Helpers for reading keyboard input, with modifiers.

import { OsType } from '@tauri-apps/plugin-os';

// Note: For now, for simplifiy, we're ignoring the SHIFT modifier,
// since it changes the key code in most cases we care about. Might
// revise this later if we decide it's useful.
export enum Modifier {
  NONE = 0,
  CTRL = 1,
  META = 2,
  SUPER = 4,
  ALL = 7,
}

const LETTERS = {
  A: 1, B: 1, C: 1, D: 1, E: 1, F: 1, G: 1, H: 1, I: 1, J: 1, K: 1, L: 1, M: 1,
  N: 1, O: 1, P: 1, Q: 1, R: 1, S: 1, T: 1, U: 1, V: 1, W: 1, X: 1, Y: 1, Z: 1,
};

const MODIFIER_KEY_NAMES = new Set([
  "Alt", "AltGr", "Shift", "Control", "Hyper", "Meta",
])

export class KeyInput {
  private _key: string;
  private _modifiers: Modifier;

  constructor(key: string, modifiers: Modifier = 0) {
    this._key = key;
    this._modifiers = modifiers;
    this.normalizeSelf();
  }

  get key(): string {
    return this._key;
  }

  get modifiers(): Modifier {
    return this._modifiers;
  }

  // Normalize the key inputs in the same way that Emacs does. This is
  // mainly an Emacs compatibility thing, and in most cases it won't
  // change the input.
  private normalizeSelf() {
    if ((this.modifiers != Modifier.NONE) && (this.key in LETTERS)) {
      this._key = this.key.toLowerCase();
    }
    switch (this.toEmacsSyntax()) {
    case "C-i":
      this._key = "Tab";
      this._modifiers = Modifier.NONE;
      break;
    case "C-m":
      this._key = "Enter";
      this._modifiers = Modifier.NONE;
      break;
    }
  }

  // Returns undefined if the keyboard event represents a modifier key
  // itself. The resulting object still has access to the original
  // event that triggered it.
  static fromEvent(event: KeyboardEvent, osType: OsType): KeyEventInput | undefined {
    if (MODIFIER_KEY_NAMES.has(event.key)) {
      return undefined;
    }
    return Object.assign(
      new KeyInput(event.key, readModifiers(event, osType)),
      { event },
    );
  }

  hasModifier(modifier: Modifier): boolean {
    return (this.modifiers & modifier) == modifier;
  }

  toEmacsSyntax(): string {
    let seq = "";
    if (this.hasModifier(Modifier.CTRL)) {
      seq += "C-";
    }
    if (this.hasModifier(Modifier.META)) {
      seq += "M-";
    }
    if (this.hasModifier(Modifier.SUPER)) {
      seq += "s-";
    }
    return seq + this.key;
  }

  static fromEmacsSyntax(input: string): KeyInput {
    let modifiers: Modifier = 0;
    while (input[1] == '-') {
      switch (input[0]) {
      case 'C':
        modifiers |= Modifier.CTRL;
        break;
      case 'M':
        modifiers |= Modifier.META;
        break;
      case 's':
        modifiers |= Modifier.SUPER;
        break;
      default:
        throw `Invalid modifier ${input[0]}`;
      }
      input = input.substring(2);
    }
    return new KeyInput(input, modifiers);
  }
}

export type KeyEventInput = KeyInput & { event: KeyboardEvent };

export function readModifiers(event: KeyboardEvent, osType: OsType): Modifier {
  let modifiers: Modifier = 0;
  if (event.ctrlKey) {
    modifiers |= Modifier.CTRL;
  }
  if (event.altKey) {
    modifiers |= (osType === "macos" ? Modifier.SUPER : Modifier.META);
  }
  if (event.metaKey) {
    modifiers |= (osType === "macos" ? Modifier.META : Modifier.SUPER);
  }
  return modifiers;
}


// Response to a keydown event.
export enum KeyResponse {
  // Pass the key input onto the parent container, outside of the
  // origin's control. Note that this does NOT imply that the original
  // responder ignored the input, only that it wishes for the parent
  // to see it.
  PASS,
  // Suppress the input and do not allow parent containers to see it.
  BLOCK,
}
