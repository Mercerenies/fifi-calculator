
import { StackUpdatedDelegate } from './stack_view.js';
import { TAURI } from './tauri_api.js';
import { GraphicsDirective, PlotDirective, ContourPlotDirective } from './tauri_api/graphics.js';
import { GLOBAL_IMAGE_CACHE } from './graphics/image_cache.js';

import Plotly from 'plotly.js-dist-min';

// StackUpdatedDelegate instance for rendering plots.
export const GRAPHICS_DELEGATE: StackUpdatedDelegate = {
  onStackUpdated(stackDiv: HTMLElement): Promise<void> {
    const graphicsElements = [...getGraphicsElements(stackDiv)];
    return Promise.all(
      graphicsElements.map((element) => renderGraphics(element as HTMLElement)),
    ).then(() => undefined);
  }
};

async function renderGraphics(element: HTMLElement): Promise<void> {
  const payload = getGraphicsPayload(element);
  if (payload == undefined) {
    console.warn('Graphics element missing payload', element);
    return;
  }
  const renderTarget = new ImageTagRenderTarget();
  renderPlotTo(payload, renderTarget);
  element.innerHTML = "";
  element.appendChild(renderTarget.imgTag);
}

export interface RenderTarget {
  getHtmlRenderTarget(): HTMLElement;
  preprocess(base64Payload: string): Promise<RenderPreprocessResult>;
  postprocess(plot: Plotly.PlotlyHTMLElement, base64Payload: string): Promise<void>;
}

export class ImageTagRenderTarget implements RenderTarget {
  readonly imgTag: HTMLImageElement;

  private useImageCache: boolean;

  constructor(args: Partial<ImageTagRenderTargetArgs> = {}) {
    if (args.imgTag == undefined) {
      this.imgTag = document.createElement('img');
      this.imgTag.className = "plotly-plot";
    } else {
      this.imgTag = args.imgTag;
    }
    this.useImageCache = args.useImageCache ?? true;
  }

  preprocess(base64Payload: string): Promise<RenderPreprocessResult> {
    if (this.useImageCache) {
      const imageUrl = GLOBAL_IMAGE_CACHE.get(base64Payload);
      if (imageUrl !== undefined) {
        this.imgTag.src = imageUrl;
        return Promise.resolve("stop");
      }
    }
    return Promise.resolve("continue");
  }

  getHtmlRenderTarget(): HTMLDivElement {
    return document.createElement('div');
  }

  async postprocess(plot: Plotly.PlotlyHTMLElement, base64Payload: string): Promise<void> {
    const image = await Plotly.toImage(plot, { width: 300, height: 300, format: 'png' });
    if (this.useImageCache) {
      GLOBAL_IMAGE_CACHE.set(base64Payload, image);
    }
    this.imgTag.src = image;
  }
}

export interface ImageTagRenderTargetArgs {
  imgTag: HTMLImageElement;
  useImageCache: boolean;
}

// Render target which directly renders to some HTML element, with
// full Plotly interactivity enabled.
export class DirectRenderTarget implements RenderTarget {
  readonly target: HTMLElement;

  constructor(target: HTMLElement) {
    this.target = target;
  }

  preprocess(): Promise<RenderPreprocessResult> {
    return Promise.resolve("continue");
  }

  getHtmlRenderTarget(): HTMLElement {
    return this.target;
  }

  postprocess(): Promise<void> {
    // Don't need to do anything after rendering.
    return Promise.resolve();
  }
}

export type RenderPreprocessResult = "stop" | "continue";

export async function renderPlotTo(
  payloadBase64: string,
  renderTarget: RenderTarget,
  layout: Partial<Plotly.Layout> = {},
  config: Partial<Plotly.Config> = {},
): Promise<void> {
  const preprocessResult = await renderTarget.preprocess(payloadBase64);
  if (preprocessResult === "stop") {
    return;
  }
  const response = await TAURI.renderGraphics(payloadBase64);
  if (response == undefined) {
    // This might be redundant, as we probably already reported
    // something to the user via the notifications interface, but it
    // isn't hurting.
    throw "Failed to render graphics";
  }
  const data = response.directives.map(directiveToTrace);
  const finalLayout = Object.assign(defaultPlotLayout(), layout);
  const finalConfig = Object.assign(defaultPlotConfig(), config);

  const plot = await Plotly.newPlot(renderTarget.getHtmlRenderTarget(), data, finalLayout, finalConfig);
  await renderTarget.postprocess(plot, payloadBase64);
}

function defaultPlotLayout(): Partial<Plotly.Layout> {
  return {
    showlegend: false,
    margin: { b: 40, l: 40, r: 40, t: 40 },
    plot_bgcolor: "rgba(0, 0, 0, 0)",
    paper_bgcolor: "rgba(0, 0, 0, 0)",
  };
}

function defaultPlotConfig(): Partial<Plotly.Config> {
  return {};
}

function directiveToTrace(directive: GraphicsDirective): Plotly.Data {
  switch (directive.type) {
  case "plot":
    return plotToTrace(directive);
  case "contourplot":
    return contourPlotToTrace(directive);
  }
}

function plotToTrace(plot: PlotDirective): Partial<Plotly.PlotData> {
  return {
    x: plot.points.map((p) => p.x),
    y: plot.points.map((p) => p.y),
    type: 'scatter',
  };
}

function contourPlotToTrace(contourPlot: ContourPlotDirective): Partial<Plotly.PlotData> {
  return {
    x: contourPlot.xValues,
    y: contourPlot.yValues,
    z: contourPlot.zValues,
    type: 'contour',
  };
}

export function getGraphicsElements<R>(element: QuerySelectable<R>): R {
  return element.querySelectorAll('[data-graphics-flag]');
}

export function getGraphicsPayload(element: HTMLElement): string | undefined {
  if (!element.dataset.graphicsFlag) {
    return undefined;
  }
  return element.dataset.graphicsPayload;
}

export interface QuerySelectable<R> {
  querySelectorAll(selector: string): R;
}
