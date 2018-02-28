import { isString } from 'substance'
import { UNKNOWN } from './CellStates'

export default class Cell {

  constructor({ id, docId, lang, source, inputs, output, value, errors, hasSideEffects, next, prev }) {
    if (!id) throw new Error("'id' is required.")
    this.id = id
    this.docId = docId
    this.lang = lang
    this.source = source
    // managed by CellGraph
    this.state = UNKNOWN
    // a set of symbols ('x', 'A1', 'A1:B10', 'doc1!x', 'sheet1!A1', 'sheet1!A1:A10', 'sheet1!foo')
    this.inputs = new Set(inputs || [])
    // an output symbol (typically only used for document cells)
    this.output = output
    // one or many CellErrors
    this.errors = errors || []
    // the last computed value
    this.value = value
    // for cells with side effects
    this.hasSideEffects = Boolean(hasSideEffects)
    // for cells in a linear model
    // this is particularly important for cells with side effects
    this.next = next
    this.prev = prev
    // used by CellGraph
    this.level = 0
    // TODO: maybe we want to keep some stats, e.g. time of last evaluation, duration of last evaluation etc.
    this.stats = {}
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