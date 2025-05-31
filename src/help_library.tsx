
// All manual pages.

import { jsx, Fragment } from './jsx.js';
import { HelpPage } from './help_manager.js';

function BinaryFunctionExplanation(): JSX.Element {
  return <span>
    <p>
      This function takes two arguments from the top of the stack by default. With
      a positive argument N, this function takes N values off the stack and reduces
      them (associating to the left) using the binary function. A numerical argument
      of zero reduces the <em>whole</em> stack.
    </p>
    <p>
      If given a negative prefix argument, the top stack element is <em>distributed</em> to
      the next N stack elements as the right-hand argument to this function.
    </p>
  </span>;
}

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

export function subgridInput(): HelpPage {
  const body = <>
    <p>
      Subgrid for inputting numbers or other datatypes.
    </p>
  </>;
  return {
    headerText: 'Input Commands',
    body,
  };
}

export function subgridModes(): HelpPage {
  const body = <>
    <p>
      Subgrid for altering calculation settings.
    </p>
  </>;
  return {
    headerText: 'Mode Commands',
    body,
  };
}

export function subgridDisplay(): HelpPage {
  const body = <>
    <p>
      Subgrid for customizing output format.
    </p>
  </>;
  return {
    headerText: 'Display Commands',
    body,
  };
}

export function subgridGraphing(): HelpPage {
  const body = <>
    <p>
      Subgrid for graphing and plotting data.
    </p>
  </>;
  return {
    headerText: 'Graphing Commands',
    body,
  };
}

export function subgridUnits(): HelpPage {
  const body = <>
    <p>
      Subgrid for working with units and dimensional algebra.
    </p>
  </>;
  return {
    headerText: 'Unit Commands',
    body,
  };
}

export function subgridStrings(): HelpPage {
  const body = <>
    <p>
      Subgrid for commands related to string manipulation.
    </p>
  </>;
  return {
    headerText: 'String Commands',
    body,
  };
}

export function subgridTranscendental(): HelpPage {
  const body = <>
    <p>
      Subgrid for transcendental mathematical functions.
    </p>
  </>;
  return {
    headerText: 'Transcendental Commands',
    body,
  };
}

export function subgridDatetime(): HelpPage {
  const body = <>
    <p>
      Subgrid for commands and functions related to datetime arithmetic.
    </p>
  </>;
  return {
    headerText: 'Datetime Commands',
    body,
  };
}

export function subgridAlgebra(): HelpPage {
  const body = <>
    <p>
      Subgrid for commands to manipulate algebraic expressions.
    </p>
  </>;
  return {
    headerText: 'Algebra Commands',
    body,
  };
}

export function subgridStorage(): HelpPage {
  const body = <>
    <p>
      Subgrid for commands related to storage and variables.
    </p>
  </>;
  return {
    headerText: 'Storage Commands',
    body,
  };
}

export function subgridVector(): HelpPage {
  const body = <>
    <p>
      Subgrid for functions that apply to vectors.
    </p>
  </>;
  return {
    headerText: 'Vector Commands',
    body,
  };
}

export function subgridMatrix(): HelpPage {
  const body = <>
    <p>
      Subgrid for functions related to matrices or linear algebra.
    </p>
  </>;
  return {
    headerText: 'Matrix Commands',
    body,
  };
}

export function subgridFormula(): HelpPage {
  const body = <>
    <p>
      Subgrid for constructors, constants, and comparison operators.
    </p>
  </>;
  return {
    headerText: 'Formula Commands',
    body,
  };
}

export function subgridVectorStats(): HelpPage {
  const body = <>
    <p>
      Subgrid for statistical and data science functions.
    </p>
  </>;
  return {
    headerText: 'Statistics Commands',
    body,
  };
}

export function inputNumerical(): HelpPage {
  const body = <>
    <p>
      Input a literal number to push onto the stack. A leading minus sign
      can be entered using the <code>_</code> hotkey. Scientific notation
      is supported using programmer notation, e.g. <code>1.0e3</code>.
    </p>
  </>;
  return {
    headerText: 'Numerical Input',
    body,
  };
}

export function inputAlgebraic(): HelpPage {
  const body = <>
    <p>
      Input an arbitrary mathematical expression to push onto the stack.
    </p>
  </>;
  return {
    headerText: 'Algebraic Input',
    body,
  };
}

export function inputString(): HelpPage {
  const body = <>
    <p>
      Input literal text to push onto the stack as a string literal.
    </p>
  </>;
  return {
    headerText: 'String Input',
    body,
  };
}

export function inputAlgebraicEdit(): HelpPage {
  const body = <>
    <p>
      Edit the top of the stack, as an algebraic expression.
    </p>
    <p>
      With a numerical argument, the nth stack element is edited.
    </p>
  </>;
  return {
    headerText: 'Algebraic Edit',
    body,
  };
}

///// TODO Fix horizontal alignment of <ul> here.
export function plus(): HelpPage {
  const body = <>
    <p>
      Adds the top two stack elements together, pushing a single result.
    </p>
    <p>
      Many of the built-in datatypes support some form of addition.
    </p>
    <ul>
      <li>Numbers (real, complex, or quaternion) are added together using the usual mathematical rules.</li>
      <li>Datetimes can be added to numbers, which are interpreted as a delta in days.</li>
      <li>Strings are concatenated.</li>
      <li>Intervals are added together as sets of real numbers.</li>
      <li>Graphics objects are concatenated into a single graphics object.</li>
    </ul>
    <p>
      Addition is broadcasted across vector arguments automatically.
    </p>
    <BinaryFunctionExplanation />
  </>;
  return {
    headerText: 'Plus',
    body,
  };
}
