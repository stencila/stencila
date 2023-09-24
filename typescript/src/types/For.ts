// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Array } from './Array';
import { Block } from './Block';
import { CodeExecutable } from './CodeExecutable';
import { Cord } from './Cord';

// Repeat a block content for each item in an array.
export class For extends CodeExecutable {
  type = "For";

  // The name to give to the variable representing each item in the iterated array
  symbol: string;

  // The content to repeat for each item
  content: Block[];

  // The content to render if there are no items
  otherwise?: Block[];

  // The content repeated for each iteration
  iterations?: Array[];

  constructor(code: Cord, programmingLanguage: string, symbol: string, content: Block[], options?: For) {
    super(code, programmingLanguage)
    if (options) Object.assign(this, options)
    this.code = code;
    this.programmingLanguage = programmingLanguage;
    this.symbol = symbol;
    this.content = content;
  }

  static from(other: For): For {
    return new For(other.code!, other.programmingLanguage!, other.symbol!, other.content!, other)
  }
}
