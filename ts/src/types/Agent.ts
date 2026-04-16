// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { CreativeWork } from "./CreativeWork.js";
import { Integer } from "./Integer.js";
import { UnsignedInteger } from "./UnsignedInteger.js";

/**
 * An AI agent definition.
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
   * Positive selection signals describing when this agent should be used.
   */
  whenToUse?: string[];

  /**
   * Negative selection signals describing when this agent should not be used.
   */
  whenNotToUse?: string[];

  /**
   * Frontmatter containing agent metadata.
   */
  frontmatter?: string;

  /**
   * The content of the agent (the Markdown body providing system instructions).
   */
  content?: Block[];

  /**
   * Model identifiers for the agent.
   */
  models?: string[];

  /**
   * Provider identifiers for the agent.
   */
  providers?: string[];

  /**
   * Model size preference for the agent.
   */
  modelSize?: string;

  /**
   * Reasoning effort level for the agent.
   */
  reasoningEffort?: string;

  /**
   * Whether to replay assistant thinking and reasoning in conversation history.
   */
  historyThinkingReplay?: string;

  /**
   * Named preset for tool output truncation limits.
   */
  truncationPreset?: string;

  /**
   * Context usage percentage that triggers proactive history compaction.
   */
  compactionTriggerPercent?: UnsignedInteger;

  /**
   * Trust level controlling how strictly the agent's operations are guarded.
   */
  trustLevel?: string;

  /**
   * Skill names this agent can use.
   */
  allowedSkills?: string[];

  /**
   * Tool names available to the agent.
   */
  allowedTools?: string[];

  /**
   * Domain allowlist for web_fetch.
   */
  allowedDomains?: string[];

  /**
   * Domain denylist for web_fetch.
   */
  disallowedDomains?: string[];

  /**
   * Whether to enable MCP tools.
   */
  enableMcp?: boolean;

  /**
   * Whether to enable MCP codemode orchestration.
   */
  enableMcpCodemode?: boolean;

  /**
   * MCP server IDs this agent is allowed to use.
   */
  allowedMcpServers?: string[];

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
