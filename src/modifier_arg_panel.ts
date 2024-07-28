
import { Signal, SignalListener } from './signal.js';
import { ModifierDelegate, ButtonModifiers, defaultModifiers } from './button_grid/modifier_delegate.js';
import { KeyEventInput, KeyResponse } from './keyboard.js';

export class ModifierArgPanel {
  private modifierArgumentsManager: ModifierArgumentsManager;
  private modifierArgPanel: HTMLElement;

  private signalListener: SignalListener<ModifiersChangedEvent> | undefined;

  constructor(args: ModifierArgPanelArgs) {
    this.modifierArgumentsManager = args.modifierArgumentsManager;
    this.modifierArgPanel = args.modifierArgPanel;
  }

  initListeners(): void {
    this.signalListener = (event) => this.onModifiersChanged(event);
    this.modifierArgumentsManager.modifiersChangedSignal.addListener(this.signalListener);
    for (const checkbox of this.getCheckboxes()) {
      checkbox.addEventListener("change", () => {
        const checkboxArg = checkbox.dataset.modifierArg!;
        if (!(checkboxArg in this.modifierArgumentsManager.values)) {
          throw new Error("Unexpected checkbox arg: " + checkboxArg);
        }
        this.modifierArgumentsManager[checkboxArg as keyof ModifierArgumentsValues] = checkbox.checked;
      });
    }
  }

  uninitListeners(): void {
    if (this.signalListener) {
      this.modifierArgumentsManager.modifiersChangedSignal.removeListener(this.signalListener);
      this.signalListener = undefined;
    }
  }

  private getCheckboxes(): NodeListOf<HTMLInputElement> {
    return this.modifierArgPanel.querySelectorAll("input[type=checkbox][data-modifier-arg]");
  }

  private onModifiersChanged(event: ModifiersChangedEvent): void {
    for (const checkbox of this.getCheckboxes()) {
      const checkboxArg = checkbox.dataset.modifierArg as keyof ModifierArgumentsValues;
      checkbox.checked = event.newModifierValues[checkboxArg];
    }
  }
}

export interface ModifierArgPanelArgs {
  modifierArgumentsManager: ModifierArgumentsManager;
  modifierArgPanel: HTMLElement;
}

export class ModifierArgumentsManager {
  private _keepModifier: boolean = false;
  readonly modifiersChangedSignal: Signal<ModifiersChangedEvent> = new Signal<ModifiersChangedEvent>();

  get keepModifier(): boolean {
    return this._keepModifier;
  }

  set keepModifier(keepModifier: boolean) {
    this._keepModifier = keepModifier;
    this.modifiersChangedSignal.emit({
      type: "modifiers-changed",
      newModifierValues: this.values,
    });
  }

  get values(): ModifierArgumentsValues {
    return {
      keepModifier: this._keepModifier
    };
  }

  get delegate(): ModifierFlagsDelegate {
    return new ModifierFlagsDelegate(this);
  }
}

// Modifier delegate that handles modifier flags (currently just the
// "Keep" modifier).
export class ModifierFlagsDelegate implements ModifierDelegate {
  private manager: ModifierArgumentsManager;

  constructor(manager: ModifierArgumentsManager) {
    this.manager = manager;
  }

  getModifiers(): ButtonModifiers {
    return Object.assign(defaultModifiers(), this.manager.values);
  }

  resetModifiers(): void {
    this.manager.keepModifier = false;
  }

  onKeyDown(input: KeyEventInput): Promise<KeyResponse> {
    switch (input.toEmacsSyntax()) {
    case "K":
      this.manager.keepModifier = !this.manager.keepModifier;
      return Promise.resolve(KeyResponse.BLOCK);
    default:
      return Promise.resolve(KeyResponse.PASS);
    }
  }
}

export interface ModifierArgumentsValues {
  keepModifier: boolean;
}

export interface ModifiersChangedEvent {
  type: "modifiers-changed";
  newModifierValues: ModifierArgumentsValues;
}
