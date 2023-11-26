// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from "./Entity.js";
import { ExecutionStatus } from "./ExecutionStatus.js";
import { PersonOrOrganizationOrSoftwareApplication } from "./PersonOrOrganizationOrSoftwareApplication.js";

/**
 * Abstract base type for a document editing instruction.
 */
export class Instruct extends Entity {
  type = "Instruct";

  /**
   * The text of the instruction.
   */
  text: string;

  /**
   * The agent that executed the instruction.
   */
  agent?: PersonOrOrganizationOrSoftwareApplication;

  /**
   * Status of the execution of the instruction.
   */
  executionStatus?: ExecutionStatus;

  constructor(text: string, options?: Partial<Instruct>) {
    super();
    if (options) Object.assign(this, options);
    this.text = text;
  }
}

/**
* Create a new `Instruct`
*/
export function instruct(text: string, options?: Partial<Instruct>): Instruct {
  return new Instruct(text, options);
}
