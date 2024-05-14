
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
  PREFIX_ARG_PANEL: 'prefix-arg-panel',
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

export function getPrefixArgPanel(): HTMLElement {
  return getElement(ElementIds.PREFIX_ARG_PANEL);
}
