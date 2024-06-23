
import { jsx, HtmlText } from './jsx.js';

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
    const listItems = [];
    for (let i = 0; i < newStackHtml.length; i++) {
      const elem = newStackHtml[i];
      const index = newStackHtml.length - i;
      const li = (
        <li class='value-stack-element' data-stack-index={index - 1} value={index}>
          <span class='value-stack-element-ordinal'>
            {index}.&nbsp;
          </span>
          <span class='value-stack-element-value'>
            <HtmlText content={elem} />
          </span>
        </li>
      );
      listItems.push(li);
    }
    const ol = (
      <ol>{listItems}</ol>
    );
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
