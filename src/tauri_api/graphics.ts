
// Types relating to the graphical subsystem of the engine.

export type SerializedGraphicsPayload = string;

export interface GraphicsResponse {
  directives: GraphicsDirective[];
}

export type GraphicsDirective = PlotDirective;

export interface PlotDirective {
  type: "plot";
  xBounds: Range<number>;
  yBounds: Range<number>;
  points: Point2D[];
}

export interface Range<T> {
  start: T;
  end: T;
}

export interface Point2D {
  x: number;
  y: number;
}
