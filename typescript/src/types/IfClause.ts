// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from './Block';
import { CodeExecutable } from './CodeExecutable';
import { Cord } from './Cord';

// A clause within a `If` node
export class IfClause extends CodeExecutable {
  type = "IfClause";

  // Whether this clause is the active clause in the parent `If` node
  isActive?: boolean;

  // The content to render if the result is truthy
  content: Block[];

  constructor(code: Cord, programmingLanguage: string, content: Block[], options?: IfClause) {
    super(code, programmingLanguage)
    if (options) Object.assign(this, options)
    this.code = code;
    this.programmingLanguage = programmingLanguage;
    this.content = content;
  }

  static from(other: IfClause): IfClause {
    return new IfClause(other.code!, other.programmingLanguage!, other.content!, other)
  }
}
