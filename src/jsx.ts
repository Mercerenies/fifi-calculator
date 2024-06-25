
// These builder functions let us use TSX syntax without a full
// front-end framework.

export type Stringy = string | number;

export interface HtmlAttrs {
  id: string;
  class: string;
};

export type DataAttrs = Record<`data-${string}`, Stringy>;

export type NestedArray<T> = T | NestedArray<T>[];

export function jsx(tag: string, attrs: Record<string, Stringy> | null, ...children: NestedArray<HTMLElement | string>[]): HTMLElement;
export function jsx<T, S>(ctor: new (attrs: T) => S, attrs: T | null): S;
export function jsx<T, S, U>(ctor: new (attrs: T & { children: U[] }) => S, attrs: T | null, ...children: NestedArray<U>[]): S;
/* eslint-disable-next-line @typescript-eslint/no-explicit-any */
export function jsx(tagOrCtor: string | (new (attrs: any) => any), attrs: Record<string, any> | null, ...children: NestedArray<any>[]): any {
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
      if (isFragment(child)) {
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

export interface _HtmlText {
  (propsOrStr: string | { content: string }): Fragment;
  new (propsOrStr: string | { content: string }): Fragment;
}

export const HtmlText = function HtmlText(propsOrStr: string | { content: string }): Fragment {
  const content = (typeof propsOrStr === 'string') ? propsOrStr : propsOrStr.content;
  const parser = new DOMParser();
  const htmlDoc = parser.parseFromString(content, 'text/html');
  return new Fragment([...htmlDoc.body.childNodes]);
} as _HtmlText;

export class Fragment {
  readonly elements: Node[];

  constructor(childrenOrProps: Node[] | { children: Node[] }) {
    if (Array.isArray(childrenOrProps)) {
      this.elements = childrenOrProps;
    } else {
      this.elements = childrenOrProps.children;
    }
  }

  querySelector(selector: string): Node | null {
    for (const element of this.elements) {
      if (element instanceof HTMLElement) {
        if (element.matches(selector)) {
          return element;
        }
        const found = element.querySelector(selector);
        if (found != null) {
          return found;
        }
      }
    }
    return null;
  }

  querySelectorAll(selector: string): Node[] {
    const elements = [];
    for (const element of this.elements) {
      if (element instanceof HTMLElement) {
        if (element.matches(selector)) {
          elements.push(element);
        }
        elements.push(...element.querySelectorAll(selector));
      }
    }
    return elements;
  }
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

export function isFragment(obj: unknown): obj is Fragment {
  return obj instanceof Fragment;
}

export function toNodes(element: JSX.Element): Node[] {
  if (element instanceof Fragment) {
    return element.elements;
  } else {
    return [element];
  }
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
    type Element = HTMLElement | Fragment;
  }
}
