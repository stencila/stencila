// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { ResearchObject } from "./ResearchObject.js";

/**
 * A request for research work, evidence, protocol execution, or another contribution.
 */
export class Request extends ResearchObject {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Request";

  constructor(content: Block[], options?: Partial<Request>) {
    super(content);
    this.type = "Request";
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `Request`
*/
export function request(content: Block[], options?: Partial<Request>): Request {
  return new Request(content, options);
}
