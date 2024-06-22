
// Manager class for displaying the current value stack.
export class StackView {
  private valueStackDiv: HTMLElement;
  private delegate: StackUpdatedDelegate;

  constructor(valueStackDiv: HTMLElement, delegate?: StackUpdatedDelegate) {
    this.valueStackDiv = valueStackDiv;
    this.delegate = delegate ?? NULL_STACK_UPDATED_DELEGATE;
  }

  async refreshStack(newStackHtml: string[]): Promise<void> {
    this.valueStackDiv.dataset.stackLength = String(newStackHtml.length);
    const ol = document.createElement("ol");
    for (let i = 0; i < newStackHtml.length; i++) {
      const elem = newStackHtml[i];
      const li = document.createElement("li");
      const ordinalSpan = document.createElement("span");
      const valueSpan = document.createElement("span");
      li.className = 'value-stack-element';
      ordinalSpan.className = 'value-stack-element-ordinal';
      valueSpan.className = 'value-stack-element-value';
      ordinalSpan.innerText = String(newStackHtml.length - i) + ". ";
      li.value = newStackHtml.length - i;
      li.dataset.stackIndex = String(newStackHtml.length - i - 1);
      valueSpan.innerHTML = elem;
      li.appendChild(ordinalSpan);
      li.appendChild(valueSpan);
      ol.appendChild(li);
    }
    const stack = this.valueStackDiv;
    stack.innerHTML = "";
    stack.appendChild(ol);
    await this.delegate.onStackUpdated(stack);
  }

  scrollToBottom(): void {
    this.valueStackDiv.scrollTo({ top: this.valueStackDiv.scrollHeight });
  }
}

export interface StackUpdatedDelegate {
  onStackUpdated(stackDiv: HTMLElement): Promise<void>;
}

export namespace StackUpdatedDelegate {
  export function several(delegates: StackUpdatedDelegate[]): StackUpdatedDelegate {
    return {
      onStackUpdated(stackDiv: HTMLElement): Promise<void> {
        return Promise.all(delegates.map((delegate) => delegate.onStackUpdated(stackDiv))).then(() => undefined);
      }
    };
  }
}

export const NULL_STACK_UPDATED_DELEGATE: StackUpdatedDelegate = {
  onStackUpdated(): Promise<void> {
    return Promise.resolve();
  }
}
