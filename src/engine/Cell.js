import { isString } from 'substance'
import { UNKNOWN, toString as cellStatusToString } from './CellStates'
import { transpile } from '../shared/expressionHelpers'
import { isExpression, qualifiedId } from '../shared/cellHelpers'

export default class Cell {

  constructor(doc, cellData) {
    const { id, lang, source, status, inputs, output, value, errors, hasSideEffects, next, prev } = cellData
    this.doc = doc

    // Attention: the given cell id is not necessarily globally unique
    // thus, we derive a unique id using the document id and the node id
    // localId is used later to be able to map back to the associated node
    // TODO: I would rather go for only one id, and always have a doc
    if (doc) {
      let docId = this.docId = doc.id
      // is the id already a qualified id?
      if (id.startsWith(docId)) {
        this.id = id
        // ATTENTION: assuming that the qualified id is joining
        // the doc id and the node id with a single character (e.g. '!')
        this.unqualifiedId = id.slice(docId.length+1)
      } else {
        this.id = qualifiedId(doc, cellData)
        this.unqualifiedId = id
      }
    } else {
      this.docId = null
      this.id = id
      this.unqualifiedId = id
    }

    this.lang = lang

    // the source code is transpiled to an object with
    // - (original) source
    // - transpiledSource
    // - symbolMapping
    // - isConstant
    this._source = this._transpile(source)

    // managed by CellGraph
    this.status = status || UNKNOWN
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

  get state() {
    console.warn('DEPRECATED: use cellState.status instead.')
    return this.status
  }

  hasOutput() {
    return Boolean(this.output)
  }

  hasValue() {
    return Boolean(this.value)
  }

  getValue() {
    return this.value
  }

  getLang() {
    return this.lang || (this.doc ? this.doc.lang : 'mini')
  }

  get source() {
    return this._source.original
  }

  set source(source) {
    this._source = this._transpile(source)
  }

  get transpiledSource() {
    return this._source.transpiled
  }

  get symbolMapping() {
    return this._source.symbolMapping
  }

  isConstant() {
    return this._source.isConstant
  }

  isSheetCell() {
    return false
  }

  toString() {
    // sheet1!A1 <- { ... source }
    let parts = []
    if (this.output) {
      parts.push(this.output)
      parts.push(' <- ')
    } else {
      parts.push(this.id)
      parts.push(': ')
    }
    parts.push(this._source.original)
    return parts.join('')
  }

  _getStatusString() {
    return cellStatusToString(this.status)
  }

  _transpile(source) {
    let original = source
    let transpiled
    let symbolMapping = {}
    let isConstant = false
    if (this.isSheetCell()) {
      let m = isExpression(source)
      if (m) {
        let L = m[0].length
        let prefix = new Array(L)
        prefix.fill(' ')
        source = prefix + source.slice(L)
        transpiled = transpile(source, symbolMapping)
      } else {
        isConstant = true
      }
    } else {
      transpiled = transpile(source, symbolMapping)
    }
    return {
      original,
      transpiled,
      symbolMapping,
      isConstant
    }
  }
}