// Generated file; do not edit. See `../rust/schema-gen` crate.

import { CodeExecutable } from "./CodeExecutable.js";
import { Cord } from "./Cord.js";

/**
 * A button.
 */
export class Button extends CodeExecutable {
  type = "Button";

  /**
   * The name of the variable associated with the button.
   */
  name: string;

  /**
   * A label for the button
   */
  label?: string;

  /**
   * Whether the button is currently disabled
   */
  isDisabled?: boolean;

  constructor(code: Cord, name: string, options?: Partial<Button>) {
    super(code);
    if (options) Object.assign(this, options);
    this.code = code;
    this.name = name;
  }
}

/**
* Create a new `Button`
*/
export function button(code: Cord, name: string, options?: Partial<Button>): Button {
  return new Button(code, name, options);
}
