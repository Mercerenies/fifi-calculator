
// Right panel of the screen.

import { ButtonGridManager, ButtonGrid } from "./button_grid.js";
import { delegates } from './button_grid/modifier_delegate.js';
import { PrefixArgStateMachine } from "./prefix_argument.js";
import { PrefixArgumentDelegate } from "./prefix_argument/prefix_delegate.js";
import { PrefixArgumentDisplay } from "./prefix_argument/display.js";
import { ModifierArgPanel, ModifierArgumentsManager } from "./modifier_arg_panel.js";
import { UndoManager } from './undo_manager.js';
import { TouchModeManager } from './touch_mode.js';
import { UiManager } from './ui_manager.js';

export class RightPanelManager {
  readonly prefixArgStateMachine: PrefixArgStateMachine;
  readonly buttonGrid: ButtonGridManager;
  readonly prefixArgDisplay: PrefixArgumentDisplay;
  readonly undoManager: UndoManager;
  readonly modifierArgManager: ModifierArgumentsManager;
  readonly modifierArgPanel: ModifierArgPanel;
  readonly touchModeManager: TouchModeManager;

  constructor(args: RightPanelArguments) {
    this.prefixArgStateMachine = new PrefixArgStateMachine();
    this.modifierArgManager = new ModifierArgumentsManager();
    this.touchModeManager = new TouchModeManager(args);

    const modifierDelegate = delegates([
      new PrefixArgumentDelegate(this.prefixArgStateMachine),
      this.modifierArgManager.delegate,
    ]);

    this.buttonGrid = new ButtonGridManager(
      args.buttonGrid,
      args.initialGrid,
      modifierDelegate,
    );
    this.prefixArgDisplay = new PrefixArgumentDisplay(
      args.prefixPanel,
      this.prefixArgStateMachine,
      args,
    );
    this.undoManager = new UndoManager(args);
    this.modifierArgPanel = new ModifierArgPanel(
      Object.assign({ modifierArgumentsManager: this.modifierArgManager }, args),
    );
  }

  initListeners() {
    this.buttonGrid.initListeners();
    this.prefixArgDisplay.initListeners();
    this.modifierArgPanel.initListeners();
    this.undoManager.initListeners();
    this.touchModeManager.initListeners();
  }
}

export interface RightPanelArguments {
  buttonGrid: HTMLElement,
  prefixPanel: HTMLElement,
  keepModifierCheckbox: HTMLInputElement,
  initialGrid: ButtonGrid,
  undoButton: HTMLButtonElement,
  redoButton: HTMLButtonElement,
  radiobuttonsDiv: HTMLElement;
  valueStackDiv: HTMLElement;
  uiManager: UiManager,
  displayBoxId?: string,
}
