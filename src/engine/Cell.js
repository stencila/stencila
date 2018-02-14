import { isString } from 'substance'
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

  clearErrors(filter) {
    if (isString(filter)) {
      const type = filter
      filter = (e) => {
        return e.type === type
      }
    }
    this.errors = this.errors.filter(e => !filter(e))
  }

  addErrors(errors) {
    this.errors = this.errors.concat(errors)
  }

  hasErrors() {
    return this.errors.length > 0
  }

  hasError(type) {
    for (let i = 0; i < this.errors.length; i++) {
      if (this.errors[i].type === type) return true
    }
    return false
  }
}
