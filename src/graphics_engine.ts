
import { StackUpdatedDelegate } from './stack_view.js';

// Engine for managing calls to the backend's graphics components for
// producing plots and graphs.
export class GraphicsEngine implements StackUpdatedDelegate {
  onStackUpdated(stackDiv: HTMLElement): Promise<void> {
    const graphicsElements = [...stackDiv.querySelectorAll('[data-graphics-flag]')];
    return Promise.all(
      graphicsElements.map((element) => this.renderGraphics(element as HTMLElement)),
    ).then(() => undefined);
  }

  private async renderGraphics(element: HTMLElement): Promise<void> {
    const payload = element.dataset.graphicsPayload;
    console.log(payload);
  }
}
