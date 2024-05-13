
// Helpers for reading keyboard input, with modifiers.

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

const MODIFIER_KEY_NAMES = new Set([
  "Alt", "AltGr", "Shift", "Control", "Hyper", "Meta",
])

export class KeyInput {
  readonly key: string;
  readonly modifiers: Modifier;

  constructor(key: string, modifiers: Modifier = 0) {
    this.key = key;
    this.modifiers = modifiers;
  }

  // Returns undefined if the keyboard event represents a modifier key
  // itself.
  static fromEvent(event: KeyboardEvent, osType: OsType): KeyInput | undefined {
    if (MODIFIER_KEY_NAMES.has(event.key)) {
      return undefined;
    }
    return new KeyInput(event.key, readModifiers(event, osType));
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

export function readModifiers(event: KeyboardEvent, osType: OsType): Modifier {
  let modifiers: Modifier = 0;
  if (event.ctrlKey) {
    modifiers |= Modifier.CTRL;
  }
  if (event.altKey) {
    modifiers |= (osType === "Darwin" ? Modifier.SUPER : Modifier.META);
  }
  if (event.metaKey) {
    modifiers |= (osType === "Darwin" ? Modifier.META : Modifier.SUPER);
  }
  return modifiers;
}
