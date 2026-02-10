// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { CreativeWork } from "./CreativeWork.js";

/**
 * An agent skill providing instructions for AI agents.
 */
export class Skill extends CreativeWork {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Skill";

  /**
   * A description of the item.
   */
  description: string;

  /**
   * The name of the item.
   */
  name: string;

  /**
   * Frontmatter containing skill metadata.
   */
  frontmatter?: string;

  /**
   * The content of the skill (the Markdown body).
   */
  content: Block[];

  /**
   * Environment requirements for the skill.
   */
  compatibility?: string;

  /**
   * Pre-approved tools for the skill.
   */
  allowedTools?: string[];

  constructor(description: string, name: string, content: Block[], options?: Partial<Skill>) {
    super();
    this.type = "Skill";
    if (options) Object.assign(this, options);
    this.description = description;
    this.name = name;
    this.content = content;
  }
}

/**
* Create a new `Skill`
*/
export function skill(description: string, name: string, content: Block[], options?: Partial<Skill>): Skill {
  return new Skill(description, name, content, options);
}
