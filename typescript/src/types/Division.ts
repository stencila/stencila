// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from './Block';
import { Cord } from './Cord';
import { Styled } from './Styled';

// Styled block content
export class Division extends Styled {
  type = "Division";

  // The content within the division
  content: Block[];

  constructor(code: Cord, content: Block[], options?: Division) {
    super(code)
    if (options) Object.assign(this, options)
    this.code = code;
    this.content = content;
  }
}
