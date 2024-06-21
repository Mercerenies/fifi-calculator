
import { StackUpdatedDelegate } from './stack_view.js';
import { TAURI } from './tauri_api.js';

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
    if (payload == undefined) {
      console.warn('Graphics element missing payload', element);
      return;
    }
    const response = await TAURI.renderGraphics(payload);
    if (response == undefined) {
      // This warning might be redundant, as we probably already
      // reported something to the user via the notifications
      // interface, but it isn't hurting.
      console.warn('Failed to render graphics');
      return;
    }
    console.log(response);
  }
}
