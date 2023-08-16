// Generated file. Do not edit; see `rust/schema-gen` crate.

import { Inline } from './Inline';
import { String } from './String';

// A hyperlink to other pages, sections within the same document, resources, or any URL.
export class Link {
  // The type of this item
  type = "Link";

  // The identifier for this item
  id?: String;

  // The textual content of the link.
  content: Inline[];

  // The target of the link.
  target: String;

  // A title for the link.
  title?: String;

  // The relation between the target and the current thing.
  rel?: String;

  constructor(content: Inline[], target: String, options?: Link) {
    if (options) Object.assign(this, options)
    this.content = content;
    this.target = target;
  }
}
