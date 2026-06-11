// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { ResearchObject } from "./ResearchObject.js";

/**
 * A question or research prompt.
 */
export class Question extends ResearchObject {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Question";

  constructor(content: Block[], options?: Partial<Question>) {
    super(content);
    this.type = "Question";
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `Question`
*/
export function question(content: Block[], options?: Partial<Question>): Question {
  return new Question(content, options);
}
