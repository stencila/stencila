// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from "./Block.js";
import { CreativeWork } from "./CreativeWork.js";

/**
 * A comment on an item, e.g on a `Article` or `SoftwareSourceCode`.
 */
export class Comment extends CreativeWork {
  type = "Comment";

  /**
   * Content of the comment, usually one or more paragraphs.
   */
  content: Block[];

  /**
   * The parent comment of this comment.
   */
  parentItem?: Comment;

  /**
   * The part or facet of the item that is being commented on.
   */
  commentAspect?: string;

  constructor(content: Block[], options?: Partial<Comment>) {
    super();
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `Comment`
*/
export function comment(content: Block[], options?: Partial<Comment>): Comment {
  return new Comment(content, options);
}
