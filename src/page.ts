
// Helpers to access known elements of the main page.

export const ElementIds = {
  INPUT_BOX: 'input-box',
  INPUT_TEXTBOX: 'input-textbox',
  INPUT_TEXTBOX_LABEL: 'input-textbox-label',
  VALUE_STACK: 'value-stack',
  NOTIFICATION_BOX: 'notification-box',
  NOTIFICATION_BOX_CLOSE_BUTTON: 'notification-box-close-button',
  NOTIFICATION_BOX_TEXT: 'notification-box-text',
  BUTTON_GRID_CONTAINER: 'button-grid-container',
  BUTTON_GRID_TOP_LABEL: 'button-grid-top-label',
  BUTTON_GRID_TOP_LABEL_CONTENTS: 'button-grid-top-label-contents',
  PREFIX_ARG_PANEL: 'prefix-arg-panel',
  MODIFIER_ARG_PANEL: 'modifier-arg-panel',
  MODIFIER_ARG_KEEP_ARG_CHECKBOX: 'modifier-arg-keep-arg',
  TOUCH_MODES: 'touch-modes',
  UNDO_BUTTON: 'undo-button',
  REDO_BUTTON: 'redo-button',
  MODELINE_BAR: 'modeline-bar',
  HELP_BUTTON: 'help-button',
};

function getElement(id: string): HTMLElement {
  const element = document.getElementById(id);
  if (element === null) {
    throw `No element with ID $id`;
  } else {
    return element;
  }
}

export function getInputBoxDiv(): HTMLDivElement {
  return getElement(ElementIds.INPUT_BOX) as HTMLDivElement;
}

export function getInputTextBox(): HTMLInputElement {
  return getElement(ElementIds.INPUT_TEXTBOX) as HTMLInputElement;
}

export function getValueStack(): HTMLElement {
  return getElement(ElementIds.VALUE_STACK);
}

export function getInputTextBoxLabel(): HTMLElement {
  return getElement(ElementIds.INPUT_TEXTBOX_LABEL);
}

export function getNotificationBoxCloseButton(): HTMLElement {
  return getElement(ElementIds.NOTIFICATION_BOX_CLOSE_BUTTON);
}

export function getNotificationBox(): HTMLElement {
  return getElement(ElementIds.NOTIFICATION_BOX);
}

export function getNotificationBoxText(): HTMLElement {
  return getElement(ElementIds.NOTIFICATION_BOX_TEXT);
}

export function getButtonGridContainer(): HTMLElement {
  return getElement(ElementIds.BUTTON_GRID_CONTAINER);
}

export function getButtonGridTopLabel(): HTMLElement {
  return getElement(ElementIds.BUTTON_GRID_TOP_LABEL);
}

export function getButtonGridTopLabelContents(): HTMLElement {
  return getElement(ElementIds.BUTTON_GRID_TOP_LABEL_CONTENTS);
}

export function getPrefixArgPanel(): HTMLElement {
  return getElement(ElementIds.PREFIX_ARG_PANEL);
}

export function getModifierArgPanel(): HTMLElement {
  return getElement(ElementIds.MODIFIER_ARG_PANEL);
}

export function getModifierArgKeepArgCheckbox(): HTMLInputElement {
  return getElement(ElementIds.MODIFIER_ARG_KEEP_ARG_CHECKBOX) as HTMLInputElement;
}

export function getTouchModesDiv(): HTMLElement {
  return getElement(ElementIds.TOUCH_MODES);
}

export function getUndoButton(): HTMLButtonElement {
  return getElement(ElementIds.UNDO_BUTTON) as HTMLButtonElement;
}

export function getRedoButton(): HTMLButtonElement {
  return getElement(ElementIds.REDO_BUTTON) as HTMLButtonElement;
}

export function getModelineBar(): HTMLElement {
  return getElement(ElementIds.MODELINE_BAR);
}

export function getHelpButton(): HTMLButtonElement {
  return getElement(ElementIds.HELP_BUTTON) as HTMLButtonElement;
}
