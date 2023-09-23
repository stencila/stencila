// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from './Block';
import { CreativeWork } from './CreativeWork';

// A comment on an item, e.g on a Article, or SoftwareSourceCode.
export class Comment extends CreativeWork {
  type = "Comment";

  // The parent comment of this comment.
  parentItem?: Comment;

  // The part or facet of the item that is being commented on.
  commentAspect?: string;

  constructor(content: Block[], options?: Comment) {
    super()
    if (options) Object.assign(this, options)
    this.content = content;
  }
}
