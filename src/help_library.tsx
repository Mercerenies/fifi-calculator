
// All manual pages.

import { jsx, Fragment } from './jsx.js';
import { HelpPage } from './help_manager.js';

function UnaryFunctionExplanation(): JSX.Element {
  return <span>
    <p>
      This function takes one argument from the top of the stack by default. With
      a positive prefix argument <code>u</code>, this function is applied (independently) to the top <code>u</code>
      stack elements. A prefix argument of zero applies the function to the <em>whole</em> stack.
    </p>
    <p>
      If given a negative prefix argument, the function is applied to a single stack element <code>u</code> down
      on the stack. That is, a prefix argument of <code>-1</code> is equivalent to no prefix argument at all, while
      a prefix argument of <code>-2</code> applies to the second stack element down, ignoring the first stack element.
    </p>
  </span>;
}

function BinaryFunctionExplanation(): JSX.Element {
  return <span>
    <p>
      This function takes two arguments from the top of the stack by default. With
      a positive prefix argument <code>u</code>, this function takes <code>u</code> values off the stack and reduces
      them (associating to the left) using the binary function. A prefix argument
      of zero reduces the <em>whole</em> stack.
    </p>
    <p>
      If given a negative prefix argument, the top stack element is <em>distributed</em> to
      the next <code>u</code> stack elements as the right-hand argument to this function.
    </p>
  </span>;
}

function BroadcastedExplanation(opts: { opName: string }): JSX.Element {
  return <span>
    <p>
      <span>{opts.opName}</span> is broadcasted across vector arguments automatically.
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

export function plus(): HelpPage {
  const body = <>
    <p>
      Adds the top two stack elements together, pushing a single result.
    </p>
    <p>
      Many of the built-in datatypes support some form of addition.
    </p>
    <ul class="help-ul">
      <li><strong>Numbers</strong> (real, complex, or quaternion) are added together using the usual mathematical rules.</li>
      <li><strong>Datetimes</strong> can be added to real numbers, in which case the latter is interpreted as a delta in days.</li>
      <li><strong>Strings</strong> are concatenated.</li>
      <li><strong>Intervals</strong> are added together as sets of real numbers.</li>
      <li><strong>Graphics objects</strong> are concatenated into a single graphics object.</li>
    </ul>
    <BroadcastedExplanation opName="Addition" />
    <BinaryFunctionExplanation />
  </>;
  return {
    headerText: 'Plus',
    body,
  };
}

export function minus(): HelpPage {
  const body = <>
    <p>
      Subtracts the top stack element from the next stack element down, pushing a single result.
    </p>
    <p>
      Many of the built-in datatypes support some form of subtraction.
    </p>
    <ul class="help-ul">
      <li><strong>Numbers</strong> (real, complex, or quaternion) are subtracted using the usual mathematical rules.</li>
      <li>Real numbers can be subtracted from <strong>datetimes</strong>, in which case they are interpreted as a delta in days.</li>
      <li>Two <strong>datetimes</strong> can be subtracted, producing a delta in days.</li>
      <li><strong>Intervals</strong> are subtracted together as sets of real numbers.</li>
    </ul>
    <BroadcastedExplanation opName="Subtraction" />
    <BinaryFunctionExplanation />
  </>;
  return {
    headerText: 'Minus',
    body,
  };
}

export function times(): HelpPage {
  const body = <>
    <p>
      Multiplies the top two stack elements together, producing a single result.
    </p>
    <p>
      <strong>Note:</strong> This multiplication operator is always treated as
      commutative and broadcasts across vectors. For non-commutative multiplication
      operations such as those over matrices or quaternions, you must use the
      <code>@</code> operator (in the Matrix Commands grid).
    </p>
    <p>
      Many of the built-in datatypes support some form of commutative multiplication.
    </p>
    <ul class="help-ul">
      <li><strong>Numbers</strong> (real or complex) are multiplied using the usual mathematical rules.</li>
      <li>Multiplying a <strong>string</strong> and a nonnegative integer repeats that string, similar to Python semantics.</li>
      <li><strong>Intervals</strong> are multiplied together as sets of real numbers.</li>
    </ul>
    <BroadcastedExplanation opName="Multiplication" />
    <BinaryFunctionExplanation />
  </>;
  return {
    headerText: 'Times (Commutative)',
    body,
  };
}

export function timesFull(): HelpPage {
  const body = <>
    <p>
      Multiplies the top two stack elements together, producing a single result. Unlike
      the "default" multiplication operation, this operator is <em>not</em> treated
      as commutative by the algebra engine.
    </p>
    <p>
      The following datatypes are supported.
    </p>
    <ul class="help-ul">
      <li><strong>Numbers</strong> (real, complex, or quaternion) are multiplied using the usual mathematical rules.</li>
      <li>Multiplication is broadcasted over <strong>vectors</strong> if and only if one of the arguments is a scalar.</li>
      <li><strong>Vector-matrix</strong> and <strong>matrix-vector</strong> multiplication is
          performed using the usual rules of mathematics. In the first case, the vector is
          treated as a row vector, and in the latter case, the vector is treated as a column
          vector.</li>
      <li><strong>Matrix-matrix</strong> multiplication is performed according to the usual rules of arithmetic.</li>
    </ul>
    <BinaryFunctionExplanation />
  </>;
  return {
    headerText: 'Times (Non-commutative)',
    body,
  };
}

export function divide(): HelpPage {
  const body = <>
    <p>
      Divides the top stack element from the next stack element down, producing a single result.
    </p>
    <p>
      In the event of division by zero, if infinity mode is on, an appropriate infinite constant is returned.
      Specifically, a directional infinity is returned if the direction can be determined,
      or NaN if not. If infinity mode is not on, then an error is returned.
    </p>
    <p>
      The following datatypes are supported.
    </p>
    <ul class="help-ul">
      <li><strong>Numbers</strong> (real or complex) are multiplied using the usual mathematical rules.</li>
      <li><strong>Intervals</strong> are divided as sets of real numbers. If the result of division would be a
          union of intervals, then the result is the smallest single interval containing that union.</li>
    </ul>
    <BroadcastedExplanation opName="Division" />
    <BinaryFunctionExplanation />
  </>;
  return {
    headerText: 'Divide',
    body,
  };
}

export function sine(): HelpPage {
  const body = <>
    <p>
      Applies the trigonometric <code>sin</code> function to the top stack element, which may be a
      real or complex number.
    </p>
    <p>
      With the HYPER flag, applies the <code>sinh</code> hyperbolic function. With the INVERSE flag, applies
      the <code>asin</code> inverse trigonometric function. With both flags, applies the <code>asinh</code> function.
    </p>
    <UnaryFunctionExplanation />
  </>;
  return {
    headerText: 'Sine',
    body,
  };
}

export function cosine(): HelpPage {
  const body = <>
    <p>
      Applies the trigonometric <code>cos</code> function to the top stack element, which may be a
      real or complex number.
    </p>
    <p>
      With the HYPER flag, applies the <code>cosh</code> hyperbolic function. With the INVERSE flag, applies
      the <code>acos</code> inverse trigonometric function. With both flags, applies the <code>acosh</code> function.
    </p>
    <UnaryFunctionExplanation />
  </>;
  return {
    headerText: 'Cosine',
    body,
  };
}

export function tangent(): HelpPage {
  const body = <>
    <p>
      Applies the trigonometric <code>tan</code> function to the top stack element, which may be a
      real or complex number.
    </p>
    <p>
      With the HYPER flag, applies the <code>tanh</code> hyperbolic function. With the INVERSE flag, applies
      the <code>atan</code> inverse trigonometric function. With both flags, applies the <code>atanh</code> function.
    </p>
    <UnaryFunctionExplanation />
  </>;
  return {
    headerText: 'Tangent',
    body,
  };
}

export function ln(): HelpPage {
  const body = <>
    <p>
      Applies the natural log function (log base <code>e</code>) to the top stack element, which may be any nonzero
      complex number. The natural log of zero is considered to be negative infinity if infinity mode is enabled, or
      an error otherwise.
    </p>
    <p>
      This function can be applied to intervals of positive real numbers and will return an interval in that case.
    </p>
    <p>
      With the HYPER flag, uses log base 10 instead. With the INVERSE flag, raises <code>e</code> to the argument
      instead of taking a logarithm. With both flags, raises 10 to the argument.
    </p>
    <UnaryFunctionExplanation />
  </>;
  return {
    headerText: 'Natural Log',
    body,
  };
}

export function log(): HelpPage {
  const body = <>
    <p>
      Takes the log of the top stack element, with respect to the second stack element down. With the INVERSE flag,
      raises the second stack element to the power of the first instead.
    </p>
    <p>
      The log value and the base can both be arbitrary nonzero real or complex numbers. Alternatively, the
      value (but not the base) may be an interval of positive real numbers.
    </p>
    <BinaryFunctionExplanation />
  </>;
  return {
    headerText: 'Natural Log',
    body,
  };
}

export function exp(): HelpPage {
  const body = <>
    <p>
      Raises Euler's constant <code>e</code> to the argument, which may be any complex number, power.
    </p>
    <p>
      This function can alternatively be applied to an interval of real numbers.
    </p>
    <p>
      With the HYPER flag, uses exponential base 10 instead. With the INVERSE flag, takes the
      natural logarithm. With both flags, takes the common (base 10) logarithm.
    </p>
    <UnaryFunctionExplanation />
  </>;
  return {
    headerText: 'Exponential Function',
    body,
  };
}

export function sqrt(): HelpPage {
  const body = <>
    <p>
      Takes the square root of the argument, which may be an arbitrary complex number.
    </p>
    <p>
      This function can alternatively be applied to an interval of nonnegative real numbers.
    </p>
    <p>
      With the HYPER flag, takes the binary logarithm (log base 2). With the INVERSE flag
      (regardless of the HYPER flag's value), raises 2 to the power of the argument.
    </p>
    <UnaryFunctionExplanation />
  </>;
  return {
    headerText: 'Square Root',
    body,
  };
}

export function substituteNumerically(): HelpPage {
  const body = <>
    <p>
      Substitutes in numerical constants and known variables to the top stack
      element and then simplifies to numerical (inexact) values. Well-known constants
      such as <code>pi</code> and <code>e</code> are substituted for their values, and
      rational numbers are replaced with their nearest floating-point equivalent, regardless
      of the current fractional mode.
    </p>
    <UnaryFunctionExplanation />
  </>;
  return {
    headerText: 'Numerical Substitute',
    body,
  };
}

export function substituteVars(): HelpPage {
  const body = <>
    <p>
      Substitutes in numerical constants and known variables to the top stack
      element. Well-known constants such as <code>pi</code> and <code>e</code> are substituted
      for their values.
    </p>
    <UnaryFunctionExplanation />
  </>;
  return {
    headerText: 'Auto Substitute',
    body,
  };
}

export function conjugate(): HelpPage {
  const body = <>
    <p>
      Computes the conjugate of the top stack element, which may be a real number, complex
      number, or quaternion. Note that this function is the identity on real numbers.
    </p>
    <BroadcastedExplanation opName="Conjugation" />
    <UnaryFunctionExplanation />
  </>;
  return {
    headerText: 'Divide',
    body,
  };
}

export function signum(): HelpPage {
  const body = <>
    <p>
      Returns the signum, or sign, of the top stack element. The signum of a real number, complex number, or
      quaternion is a number of magnitude 1 in the same direction as the original. For convenience, the signum
      of zero is treated as zero. Infinity constants are handled similarly. Unsigned infinities (such as NaN)
      result in an error.
    </p>
    <p>
      If applied to a vector, returns the normalized vector. If given a zero vector, returns the same zero vector.
    </p>
    <UnaryFunctionExplanation />
  </>;
  return {
    headerText: 'Signum',
    body,
  };
}

export function complexArg(): HelpPage {
  const body = <>
    <p>
      Returns the argument, or phase, of a complex number, in radians.
    </p>
    <p>
      The argument of a signed infinity is the direction of that infinity in radians. The argument of
      an unsigned infinity is NaN.
    </p>
    <BroadcastedExplanation opName="Complex argument" />
    <UnaryFunctionExplanation />
  </>;
  return {
    headerText: 'Complex Argument',
    body,
  };
}

export function realPart(): HelpPage {
  const body = <>
    <p>
      Returns the real part of a number, which may be a complex number or a quaternion.
    </p>
    <BroadcastedExplanation opName="Real part" />
    <UnaryFunctionExplanation />
  </>;
  return {
    headerText: 'Real Part',
    body,
  };
}

export function imagPart(): HelpPage {
  const body = <>
    <p>
      Returns the imaginary part of a number, which may be a complex number
      or a quaternion. In the case of a quaternion, the imaginary part is defined
      to be the real coefficient of the <code>i</code>, or second, term of the
      quaternion.
    </p>
    <BroadcastedExplanation opName="Imaginary part" />
    <UnaryFunctionExplanation />
  </>;
  return {
    headerText: 'Imaginary Part',
    body,
  };
}

export function minFunction(): HelpPage {
  const body = <>
    <p>
      Returns the smaller of the top two stack elements. Both arguments must be of the same
      type. Supported types include:
    </p>
    <ul class="help-ul">
      <li>Real numbers, including positive and negative infinity</li>
      <li>Datetime values</li>
      <li>Strings</li>
    </ul>
    <BinaryFunctionExplanation />
  </>;
  return {
    headerText: 'Minimum',
    body,
  };
}

export function maxFunction(): HelpPage {
  const body = <>
    <p>
      Returns the larger of the top two stack elements. Both arguments must be of the same
      type. Supported types include:
    </p>
    <ul class="help-ul">
      <li>Real numbers, including positive and negative infinity</li>
      <li>Datetime values</li>
      <li>Strings</li>
    </ul>
    <BinaryFunctionExplanation />
  </>;
  return {
    headerText: 'Maximum',
    body,
  };
}
