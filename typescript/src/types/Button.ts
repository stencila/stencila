// Generated file; do not edit. See `../rust/schema-gen` crate.

import { CodeExecutable } from './CodeExecutable';
import { Cord } from './Cord';

// A button.
export class Button extends CodeExecutable {
  type = "Button";

  // The name of the variable associated with the button.
  name: string;

  // A label for the button
  label?: string;

  // Whether the button is currently disabled
  isDisabled?: boolean;

  constructor(code: Cord, programmingLanguage: string, name: string, options?: Button) {
    super(code, programmingLanguage)
    if (options) Object.assign(this, options)
    this.code = code;
    this.programmingLanguage = programmingLanguage;
    this.name = name;
  }

  static from(other: Button): Button {
    return new Button(other.code!, other.programmingLanguage!, other.name!, other)
  }
}
