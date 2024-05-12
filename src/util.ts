
export function svg(source: string, opts: Partial<SvgOptions> = {}): HTMLElement {
  const img = document.createElement("img");
  img.src = source;
  img.className = opts.className || "svg-icon";
  img.alt = opts.alt || "";
  return img;
}

export interface SvgOptions {
  className: string;
  alt: string;
}
