export class CellError extends Error {
  constructor(msg, details) {
    super(msg)
    this.details = details
  }
}

export class SyntaxError extends CellError {
  get type() { return 'engine' }
  get name() { return 'syntax' }
}

export class ContextError extends CellError {
  get type() { return 'engine' }
  get name() { return 'context' }
}

export class GraphError extends CellError {
  get type() { return 'graph' }
}

export class UnresolvedInputError extends GraphError {
  get name() { return 'unresolved' }
}

export class CyclicDependencyError extends GraphError {
  get trace() {
    return this.details.trace
  }
  get name() { return 'cyclic'}
}

export class OutputCollisionError extends GraphError {
  get name() { return 'collision'}
}

export class RuntimeError extends CellError {
  get type() { return 'runtime' }
  get name() { return 'runtime' }
}

export class ValidationError extends CellError {
  get type() { return 'runtime' }
  get name() { return 'validation' }
}
