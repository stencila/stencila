// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { CreativeWork } from "./CreativeWork.js";
import { Integer } from "./Integer.js";

/**
 * An agent definition specifying model, tools, and behavioral configuration.
 */
export class Agent extends CreativeWork {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Agent";

  /**
   * A description of the item.
   */
  description: string;

  /**
   * The name of the agent.
   */
  declare name: string;

  /**
   * Frontmatter containing agent metadata.
   */
  frontmatter?: string;

  /**
   * The content of the agent (the Markdown body providing system instructions).
   */
  content?: Block[];

  /**
   * Model identifier for the agent.
   */
  model?: string;

  /**
   * Provider identifier for the agent.
   */
  provider?: string;

  /**
   * Skill names this agent can use.
   */
  allowedSkills?: string[];

  /**
   * Tool names available to the agent.
   */
  allowedTools?: string[];

  /**
   * Reasoning effort level for the agent.
   */
  reasoningEffort?: string;

  /**
   * Maximum conversation turns (0 = unlimited).
   */
  maxTurns?: Integer;

  /**
   * Default timeout for tool execution in seconds.
   */
  toolTimeout?: Integer;

  /**
   * Maximum tool-call rounds per user input.
   */
  maxToolRounds?: Integer;

  /**
   * Maximum subagent nesting depth.
   */
  maxSubagentDepth?: Integer;

  /**
   * Environment requirements for the agent.
   */
  compatibility?: string;

  /**
   * Whether to enable MCP tools.
   */
  enableMcp?: boolean;

  /**
   * Whether to enable codemode orchestration.
   */
  enableCodemode?: boolean;

  constructor(description: string, name: string, options?: Partial<Agent>) {
    super();
    this.type = "Agent";
    if (options) Object.assign(this, options);
    this.description = description;
    this.name = name;
  }
}

/**
* Create a new `Agent`
*/
export function agent(description: string, name: string, options?: Partial<Agent>): Agent {
  return new Agent(description, name, options);
}
