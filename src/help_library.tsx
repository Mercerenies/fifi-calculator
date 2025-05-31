
// All manual pages.

import { jsx, Fragment } from './jsx.js';
import { HelpPage } from './help_manager.js';

export function backButton(): HelpPage {
  const body = <>
    <p>
      Back to the previous button grid.
    </p>
  </>;
  return {
    headerText: 'Back',
    body,
  };
}
