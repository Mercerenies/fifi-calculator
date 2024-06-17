
// Manager class for displaying the current value stack.
export class StackView {
  private valueStackDiv: HTMLElement;

  constructor(valueStackDiv: HTMLElement) {
    this.valueStackDiv = valueStackDiv;
  }

  refreshStack(newStackHtml: string[]): void {
    const ol = document.createElement("ol");
    for (let i = 0; i < newStackHtml.length; i++) {
      const elem = newStackHtml[i];
      const li = document.createElement("li");
      li.className = 'value-stack-element';
      li.value = newStackHtml.length - i;
      li.dataset.stackIndex = String(newStackHtml.length - i - 1);
      li.innerHTML = elem;
      ol.appendChild(li);
    }
    const stack = this.valueStackDiv;
    stack.innerHTML = "";
    stack.appendChild(ol);
  }

  scrollToBottom(): void {
    this.valueStackDiv.scrollTo({ top: this.valueStackDiv.scrollHeight });
  }
}
