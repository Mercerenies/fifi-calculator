
export class NotificationManager {
  private notificationBox: HTMLElement;
  private textNode: HTMLElement;
  private closeButton: HTMLElement | null;

  constructor(notificationBox: HTMLElement) {
    this.notificationBox = notificationBox;
    this.closeButton = notificationBox.querySelector(`.${CLOSE_BUTTON_CSS_CLASS}`);
    const textNode = notificationBox.querySelector(`.${TEXT_CSS_CLASS}`);
    if (textNode === null) {
      throw "No text node found in notification box";
    }
    this.textNode = textNode as HTMLElement;
  }

  initListeners(): void {
    if (this.closeButton !== null) {
      this.closeButton.addEventListener("click", () => {
        this.notificationBox.style.display = "none";
      });
    }
  }

  show(htmlText: string): void {
    this.textNode.innerHTML = htmlText;
    this.notificationBox.style.display = "block";
  }
}

const CLOSE_BUTTON_CSS_CLASS = "notification-box-close-button";
const TEXT_CSS_CLASS = "notification-box-text";