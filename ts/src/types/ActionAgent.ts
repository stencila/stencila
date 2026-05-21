// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { hydrate } from "../hydrate.js";

import { type Agent } from "./Agent.js";
import { type Organization } from "./Organization.js";
import { type Person } from "./Person.js";
import { type SoftwareApplication } from "./SoftwareApplication.js";

/**
 * A human, organization, software application, or Stencila AI agent that performs, provides, or participates in an action.
 */
export type ActionAgent =
  Person |
  Organization |
  SoftwareApplication |
  Agent;

/**
 * Create a `ActionAgent` from an object
 */
export function actionAgent(other: ActionAgent): ActionAgent {
  switch(other.type) {
    case "Person":
    case "Organization":
    case "SoftwareApplication":
    case "Agent":
      return hydrate(other) as ActionAgent
    default:
      // @ts-expect-error that this can never happen because this function may be used in weakly-typed JavaScript
      throw new Error(`Unexpected type for ActionAgent: ${other.type}`);
  }
}
