// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { CreativeWork } from "./CreativeWork.js";

/**
 * A comment on an item.
 */
export class Comment extends CreativeWork {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Comment";

  /**
   * Content of the comment, usually one or more paragraphs.
   */
  content: Block[];

  /**
   * The parent comment of this comment.
   */
  parentItem?: Comment;

  /**
   * The location where the commented region begins.
   */
  startLocation?: string;

  /**
   * The location where the commented region ends.
   */
  endLocation?: string;

  constructor(content: Block[], options?: Partial<Comment>) {
    super();
    this.type = "Comment";
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
