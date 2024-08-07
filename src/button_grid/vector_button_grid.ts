
import { ButtonGrid, GridCell } from "../button_grid.js";
import { backButton, DispatchButton,
         SubcommandDispatchButton, DoubleSubcommandDispatchButton, GotoButton } from './button.js';
import { UnsignedNumberedButton } from './button/numbered.js';
import { svg } from '../util.js';

function magnifyingLensSvg(): HTMLElement {
  return svg('assets/magnifying.svg', {alt: "search"});
}

function barGraphSvg(): HTMLElement {
  return svg('assets/bar_graph.svg', {alt: "statistics"});
}

export class VectorButtonGrid extends ButtonGrid {
  readonly rows: readonly (readonly GridCell[])[];

  private rootGrid: ButtonGrid;

  constructor(rootGrid: ButtonGrid, subgrids: VectorButtonGridSubgrids) {
    super();
    this.rootGrid = rootGrid;
    this.rows = this.initRows(subgrids);
  }

  private initRows(subgrids: VectorButtonGridSubgrids): GridCell[][] {
    return [
      [
        new DispatchButton("p", "pack", "p"),
        new DispatchButton("u", "unpack", "u"),
        new DispatchButton("<math><mi>ι</mi></math>", "iota", "x"),
        new DispatchButton("<math><mo>*</mo></math>", "repeat", "b"),
        new DispatchButton("sub", "subvector", "s"),
      ],
      [
        new DispatchButton("++", "vconcat", "|"),
        new DispatchButton("::", "cons", "k"),
        new DispatchButton("ɹ", "reverse", "v"),
        new DispatchButton("⌿", "vmask", "m"),
      ],
      [
        new DispatchButton("<math><mo>&times;</mo></math>", "cross", "C"),
        new SubcommandDispatchButton("A", "vapply", "A"),
        new SubcommandDispatchButton("M", "vmap", "M"),
        new SubcommandDispatchButton("R", "vreduce", "R"),
        new SubcommandDispatchButton("U", "vaccum", "U"),
      ],
      [
        new DispatchButton("len", "length", "l"),
        new UnsignedNumberedButton("$", "arrange", "a", "Width:"),
        new DispatchButton(magnifyingLensSvg(), "find", "f"),
        new DispatchButton("Az", "sort", "S"),
        new DispatchButton("⍋", "grade", "G"),
      ],
      [
        new DispatchButton("<small><math><mrow><mo>|</mo><mi>·</mi><mo>|</mo></mrow></math></small>", "norm", "N"),
        new SubcommandDispatchButton("<math><mo>&otimes;</mo></math>", "outerprod", "O"),
        innerProductButton(),
      ],
      [
        backButton(this.rootGrid),
        new GotoButton(barGraphSvg(), "V", subgrids.vectorStats),
      ],
    ];
  }
}

export interface VectorButtonGridSubgrids {
  vectorStats: ButtonGrid,
}

function innerProductButton(): DoubleSubcommandDispatchButton {
  return new DoubleSubcommandDispatchButton("<math><mi>·</mi></math>", "innerprod", "I", {
    firstLabelHTML: "Entering <tt>&lt;×&gt;</tt> subcommand...",
    secondLabelHTML: "Entering <tt>&lt;+&gt;</tt> subcommand...",
  });
}
