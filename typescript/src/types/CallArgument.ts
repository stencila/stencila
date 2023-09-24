// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Parameter } from './Parameter';

// The value of a `Parameter` to call a document with
export class CallArgument extends Parameter {
  type = "CallArgument";

  // The code to be evaluated for the parameter.
  code: string;

  // The programming language of the code.
  programmingLanguage: string;

  // Whether the programming language of the code should be guessed based on syntax and variables used
  guessLanguage?: boolean;

  constructor(name: string, code: string, programmingLanguage: string, options?: CallArgument) {
    super(name)
    if (options) Object.assign(this, options)
    this.name = name;
    this.code = code;
    this.programmingLanguage = programmingLanguage;
  }

  static from(other: CallArgument): CallArgument {
    return new CallArgument(other.name!, other.code!, other.programmingLanguage!, other)
  }
}
