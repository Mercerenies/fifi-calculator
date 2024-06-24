
// These builder functions let us use TSX syntax without a full
// front-end framework.

export type Stringy = string | number;

export interface HtmlAttrs {
  id: string;
  class: string;
};

export interface DataAttrs {
  [key: `data-${string}`]: Stringy;
}

export type NestedArray<T> = T | NestedArray<T>[];

export function jsx(tag: string, attrs: Record<string, Stringy> | null, ...children: NestedArray<HTMLElement | string>[]): HTMLElement;
export function jsx<T, S>(ctor: { new (attrs: T): S }, attrs: T | null): S;
export function jsx<T, S, U>(ctor: { new (attrs: T & { children: U[] }): S }, attrs: T | null, ...children: NestedArray<U>[]): S;
export function jsx(tagOrCtor: string | { new (attrs: any): any }, attrs: Record<string, any> | null, ...children: NestedArray<any>[]): any {
  const flattenedChildren = flatten(children);
  if (typeof tagOrCtor === 'string') {
    const element = document.createElement(tagOrCtor);
    if (attrs != null) {
      for (const attr in attrs) {
        if (attr.startsWith("data-")) {
          element.dataset[dataify(attr.slice(5))] = String(attrs[attr]);
        } else {
          element.setAttribute(attr, String(attrs[attr]));
        }
      }
    }
    for (const child of flattenedChildren) {
      if (child.__isFragment) {
        element.append(...child.elements);
      } else {
        element.appendChild(nodify(child));
      }
    }
    return element;
  } else {
    const props = attrs ?? {};
    if (children.length > 0) {
      props.children = flattenedChildren;
    }
    return new tagOrCtor(props);
  }
}

function nodify(element: string | number | Node): Node {
  if (typeof element === 'string' || typeof element === 'number') {
    return document.createTextNode(String(element));
  } else {
    return element;
  }
}

function dataify(key: string): string {
  return key.replace(/-([a-z])/, (_, letter) => letter.toUpperCase());
}

export function HtmlText(props: { content: string }): Fragment {
  const parser = new DOMParser();
  const htmlDoc = parser.parseFromString(props.content, 'text/html');
  return {
    __isFragment: true,
    elements: [...htmlDoc.body.childNodes],
  };
}

export function Fragment(props: { children: Node[] }): Fragment {
  return {
    __isFragment: true,
    elements: props.children,
  };
}

export interface Fragment {
  readonly __isFragment: true;
  readonly elements: Node[];
}

export function flatten<T>(arr: NestedArray<T>): T[] {
  if (Array.isArray(arr)) {
    const result = [];
    for (const item of arr) {
      result.push(...flatten(item));
    }
    return result;
  } else {
    return [arr];
  }
}

export function isFragment(obj: any): obj is Fragment {
  return typeof obj === 'object' && obj.__isFragment;
}

declare global {
  namespace JSX {
    interface IntrinsicElements {
      ol: Partial<HtmlAttrs & DataAttrs>,
      li: Partial<HtmlAttrs & DataAttrs & { value: Stringy }>,
      span: Partial<HtmlAttrs & DataAttrs>,
      div: Partial<HtmlAttrs & DataAttrs>,
      button: Partial<HtmlAttrs & DataAttrs>,
      header: Partial<HtmlAttrs & DataAttrs>,
      main: Partial<HtmlAttrs & DataAttrs>,
      footer: Partial<HtmlAttrs & DataAttrs>,
    }
  }
}
