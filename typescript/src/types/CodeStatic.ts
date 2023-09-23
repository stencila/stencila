// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Cord } from './Cord';
import { Entity } from './Entity';

// Abstract base type for non-executable code nodes (e.g. `CodeBlock`).
export class CodeStatic extends Entity {
  type = "CodeStatic";

  // The code.
  code: Cord;

  // The programming language of the code.
  programmingLanguage?: string;

  constructor(code: Cord, options?: CodeStatic) {
    super()
    if (options) Object.assign(this, options)
    this.code = code;
  }
}
