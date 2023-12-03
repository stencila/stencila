// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from "./Block.js";
import { Executable } from "./Executable.js";
import { FormDeriveAction } from "./FormDeriveAction.js";
import { IntegerOrString } from "./IntegerOrString.js";

/**
 * A form to batch updates in document parameters.
 */
export class Form extends Executable {
  type = "Form";

  /**
   * The content within the form, usually containing at least one `Parameter`.
   */
  content: Block[];

  /**
   * The dotted path to the object (e.g a database table) that the form should be derived from
   */
  deriveFrom?: string;

  /**
   * The action (create, update or delete) to derive for the form
   */
  deriveAction?: FormDeriveAction;

  /**
   * An identifier for the item to be the target of Update or Delete actions
   */
  deriveItem?: IntegerOrString;

  constructor(content: Block[], options?: Partial<Form>) {
    super();
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `Form`
*/
export function form(content: Block[], options?: Partial<Form>): Form {
  return new Form(content, options);
}
