
export class HelpManager {
  private helpButton: HTMLButtonElement;

  constructor(args: HelpManagerConstructorArgs) {
    this.helpButton = args.helpButton;
  }

  initListeners(): void {
    this.helpButton.addEventListener("click", () => this.onHelpButtonClicked());
  }

  private onHelpButtonClicked(): void {
    console.log("Help button clicked!");
  }
}

export interface HelpManagerConstructorArgs {
  readonly helpButton: HTMLButtonElement;
}
