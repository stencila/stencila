// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from './Inline';

// A hyperlink to other pages, sections within the same document, resources, or any URL.
export class Link {
  type = "Link";

  // The identifier for this item
  id?: string;

  // The textual content of the link.
  content: Inline[];

  // The target of the link.
  target: string;

  // A title for the link.
  title?: string;

  // The relation between the target and the current thing.
  rel?: string;

  constructor(content: Inline[], target: string, options?: Link) {
    if (options) Object.assign(this, options)
    this.content = content;
    this.target = target;
  }
}
