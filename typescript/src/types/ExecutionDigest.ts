// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Number } from './Number';

// A digest of the execution state of a node.
export class ExecutionDigest {
  // A digest of the state of a node.
  stateDigest: Number;

  // A digest of the "semantic intent" of the resource with respect to the dependency graph
  semanticDigest: Number;

  // A digest of the semantic digests the dependencies of a resource.
  dependenciesDigest: Number;

  // A count of the number of execution dependencies that are stale
  dependenciesStale: Number;

  // A count of the number of execution dependencies that failed
  dependenciesFailed: Number;

  constructor(stateDigest: Number, semanticDigest: Number, dependenciesDigest: Number, dependenciesStale: Number, dependenciesFailed: Number, options?: ExecutionDigest) {
    if (options) Object.assign(this, options)
    this.stateDigest = stateDigest;
    this.semanticDigest = semanticDigest;
    this.dependenciesDigest = dependenciesDigest;
    this.dependenciesStale = dependenciesStale;
    this.dependenciesFailed = dependenciesFailed;
  }
}
