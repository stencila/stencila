// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from "./Entity.js";
import { UnsignedInteger } from "./UnsignedInteger.js";

/**
 * A digest of the content, semantics and dependencies of an executable node.
 */
export class CompilationDigest extends Entity {
  type = "CompilationDigest";

  /**
   * A digest of the state of a node.
   */
  stateDigest: UnsignedInteger;

  /**
   * A digest of the semantics of the node with respect to the dependency graph.
   */
  semanticDigest?: UnsignedInteger;

  /**
   * A digest of the semantic digests of the dependencies of a node.
   */
  dependenciesDigest?: UnsignedInteger;

  /**
   * A count of the number of dependencies that are stale.
   */
  dependenciesStale?: UnsignedInteger;

  /**
   * A count of the number of dependencies that failed.
   */
  dependenciesFailed?: UnsignedInteger;

  constructor(stateDigest: UnsignedInteger, options?: Partial<CompilationDigest>) {
    super();
    if (options) Object.assign(this, options);
    this.stateDigest = stateDigest;
  }
}

/**
* Create a new `CompilationDigest`
*/
export function compilationDigest(stateDigest: UnsignedInteger, options?: Partial<CompilationDigest>): CompilationDigest {
  return new CompilationDigest(stateDigest, options);
}
