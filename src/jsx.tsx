
// JSX helpers

import { ReactElement } from 'jsx-dom';

// Interpret strings as HTML, passes through other elements.
export function HtmlText(opts: { children: ReactElement | string }): ReactElement {
  if (typeof opts.children === 'string') {
    return <span innerHTML={opts.children} />;
  } else if (opts.children instanceof DocumentFragment) {
    return opts.children; // Weird but probably okay to not clone in this case.
  } else {
    return opts.children.cloneNode(true);
  }
}

