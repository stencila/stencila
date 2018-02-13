export class CellError extends Error {
  constructor(msg, details) {
    super(msg)
    this.details = details
  }
}

export class SyntaxError extends Error {
  get type() { return 'engine' }
}

export class GraphError extends Error {
  get type() { return 'graph' }
}

export class RuntimeError extends Error {
  get type() { return 'runtime' }
}

export class ValidationError extends Error {
  get type() { return 'validation' }
}
