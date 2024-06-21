
import { StackUpdatedDelegate } from './stack_view.js';
import { TAURI } from './tauri_api.js';
import { PlotDirective } from './tauri_api/graphics.js';

import Plotly from 'plotly.js-dist-min';

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
    const data = await Promise.all(response.directives.map(async (directive) => {
      switch (directive.type) {
      case "plot":
        return await this.plotToTrace(directive);
      }
    }));
    const layout = {
      showlegend: false,
      margin: { b: 40, l: 40, r: 40, t: 40 },
      plot_bgcolor: "rgba(0, 0, 0, 0)",
      paper_bgcolor: "rgba(0, 0, 0, 0)",
    } as const;

    const div = document.createElement('div');
    const plot = await Plotly.newPlot(div, data, layout);
    const image = await Plotly.toImage(plot, { width: 300, height: 300, format: 'png' });
    const imgTag = document.createElement('img');
    imgTag.className = "plotly-plot";
    imgTag.src = image;
    element.innerHTML = "";
    element.appendChild(imgTag);
  }

  private async plotToTrace(plot: PlotDirective): Promise<ScatterTrace> {
    return {
      x: plot.points.map((p) => p.x),
      y: plot.points.map((p) => p.y),
      type: 'scatter',
    };
  }
}

export interface ScatterTrace {
  x: number[];
  y: number[];
  type: 'scatter';
}
