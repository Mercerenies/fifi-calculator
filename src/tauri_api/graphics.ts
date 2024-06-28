
// Types relating to the graphical subsystem of the engine.

export type SerializedGraphicsPayload = string;

export interface GraphicsResponse {
  directives: GraphicsDirective[];
}

export type GraphicsDirective = PlotDirective | ContourPlotDirective;

export interface PlotDirective {
  type: "plot";
  points: Point2D[];
}

export interface ContourPlotDirective {
  type: "contourplot";
  xValues: number[];
  yValues: number[];
  zValues: number[][];
}

export interface Range<T> {
  start: T;
  end: T;
}

export interface Point2D {
  x: number;
  y: number;
}
