// Generated file; do not edit. See `../rust/schema-gen` crate.

import { CallArgument } from './CallArgument';
import { Include } from './Include';

// Call another document, optionally with arguments, and include its executed content.
export class Call extends Include {
  type = "Call";

  // The value of the source document's parameters to call it with
  arguments: CallArgument[];

  constructor(source: string, args: CallArgument[], options?: Call) {
    super(source)
    if (options) Object.assign(this, options)
    this.source = source;
    this.arguments = args;
  }
}
