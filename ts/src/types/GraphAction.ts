// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { hydrate } from "../hydrate.js";

import { type Action } from "./Action.js";
import { type ConvertAction } from "./ConvertAction.js";
import { type CreateAction } from "./CreateAction.js";
import { type ExecuteAction } from "./ExecuteAction.js";

/**
 * An action associated with a graph edge.
 */
export type GraphAction =
  Action |
  CreateAction |
  ConvertAction |
  ExecuteAction;

/**
 * Create a `GraphAction` from an object
 */
export function graphAction(other: GraphAction): GraphAction {
  switch(other.type) {
    case "Action":
    case "CreateAction":
    case "ConvertAction":
    case "ExecuteAction":
      return hydrate(other) as GraphAction
    default:
      // @ts-expect-error that this can never happen because this function may be used in weakly-typed JavaScript
      throw new Error(`Unexpected type for GraphAction: ${other.type}`);
  }
}
