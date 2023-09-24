// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from './Entity';

// A digest of the execution state of a node.
export class ExecutionDigest extends Entity {
  type = "ExecutionDigest";

  // A digest of the state of a node.
  stateDigest: number;

  // A digest of the "semantic intent" of the resource with respect to the dependency graph
  semanticDigest: number;

  // A digest of the semantic digests the dependencies of a resource.
  dependenciesDigest: number;

  // A count of the number of execution dependencies that are stale
  dependenciesStale: number;

  // A count of the number of execution dependencies that failed
  dependenciesFailed: number;

  constructor(stateDigest: number, semanticDigest: number, dependenciesDigest: number, dependenciesStale: number, dependenciesFailed: number, options?: ExecutionDigest) {
    super()
    if (options) Object.assign(this, options)
    this.stateDigest = stateDigest;
    this.semanticDigest = semanticDigest;
    this.dependenciesDigest = dependenciesDigest;
    this.dependenciesStale = dependenciesStale;
    this.dependenciesFailed = dependenciesFailed;
  }

  static from(other: ExecutionDigest): ExecutionDigest {
    return new ExecutionDigest(other.stateDigest!, other.semanticDigest!, other.dependenciesDigest!, other.dependenciesStale!, other.dependenciesFailed!, other)
  }
}
