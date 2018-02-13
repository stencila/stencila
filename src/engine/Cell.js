import { UNKNOWN } from './CellStates'

export default class Cell {

  constructor({ id, inputs, output }) {
    if (!id) throw new Error("'id' is required.")
    this.id = id
    this.state = UNKNOWN
    this.errors = []
    this.inputs = new Set(inputs || [])
    this.output = output || null
    this.value = undefined

    // used by CellGraph
    this.level = 0
  }

  clearErrors(type) {
    this.errors = this.errors.filter(e => e.type !== type)
  }

  addErrors(errors) {
    this.errors = this.errors.concat(errors)
  }

  hasErrors() {
    return this.errors.length > 0
  }
}
