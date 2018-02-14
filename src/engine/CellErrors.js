export class CellError extends Error {
  constructor(msg, details) {
    super(msg)
    this.details = details
  }
}

export class SyntaxError extends CellError {
  get type() { return 'engine' }
}

export class GraphError extends CellError {
  get type() { return 'graph' }
}

export class UnresolvedInputError extends GraphError {}

export class CyclicDependencyError extends GraphError {
  get trace() {
    return this.details.trace
  }
}

export class OutputCollisionError extends GraphError {}

export class RuntimeError extends CellError {
  get type() { return 'runtime' }
}

export class ValidationError extends CellError {
  get type() { return 'validation' }
}
