
import { SubcommandId } from '../tauri_api.js';

// The behavior of a button or grid cell when we try to use it as a
// subcommand. Valid behaviors are:
//
// * IsSubcommand: The button can be treated as a subcommand.
//
// * "pass": The button should fire like normal, even if we're
// inputting a subcommand.
//
// * "invalid": The button is NOT valid as a subcommand.
export type SubcommandBehavior = IsSubcommand | "pass" | "invalid";

export class IsSubcommand {
  subcommand: SubcommandId;

  constructor(subcommand: SubcommandId) {
    this.subcommand = subcommand;
  }
}
