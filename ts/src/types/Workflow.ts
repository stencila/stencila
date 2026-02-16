// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { CreativeWork } from "./CreativeWork.js";
import { Integer } from "./Integer.js";

/**
 * A workflow pipeline definition using Graphviz DOT syntax to orchestrate multi-stage AI tasks.
 */
export class Workflow extends CreativeWork {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Workflow";

  /**
   * A description of the item.
   */
  description: string;

  /**
   * The name of the workflow.
   */
  declare name: string;

  /**
   * Frontmatter containing workflow metadata.
   */
  frontmatter?: string;

  /**
   * The content of the workflow (Markdown body containing the DOT pipeline and documentation).
   */
  content?: Block[];

  /**
   * The raw DOT source defining the pipeline digraph.
   */
  pipeline?: string;

  /**
   * The high-level goal for the pipeline.
   */
  goal?: string;

  /**
   * CSS-like stylesheet for supplementary per-node LLM model and provider overrides.
   */
  modelStylesheet?: string;

  /**
   * Global retry ceiling for nodes that omit max_retries.
   */
  defaultMaxRetry?: Integer;

  /**
   * Node ID to jump to if exit is reached with unsatisfied goal gates.
   */
  retryTarget?: string;

  /**
   * Secondary jump target if retryTarget is missing or invalid.
   */
  fallbackRetryTarget?: string;

  /**
   * Default context fidelity mode for LLM sessions.
   */
  defaultFidelity?: string;

  constructor(description: string, name: string, options?: Partial<Workflow>) {
    super();
    this.type = "Workflow";
    if (options) Object.assign(this, options);
    this.description = description;
    this.name = name;
  }
}

/**
* Create a new `Workflow`
*/
export function workflow(description: string, name: string, options?: Partial<Workflow>): Workflow {
  return new Workflow(description, name, options);
}
