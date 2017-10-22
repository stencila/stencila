import { forEach, uuid } from 'substance'
import CellState from '../engine/CellState'

const CELL_TYPES = {
  'cell': true,
  'inline-cell': true
}

const INPUT_TYPES = {
  'select': true,
  'range-input': true
}

export default class DocumentEngineAdapter {

  constructor(editorSession) {
    this.editorSession = editorSession
    this.doc = editorSession.getDocument()
    // Note: this id is used internally to
    // lookup variables and cells
    if (!this.doc.UUID) {
      this.doc.UUID = uuid()
    }
    // set by Engine
    this.engine = null
  }

  connect(engine) {
    if (this.engine) throw new Error('This resource is already connected to an engine.')
    this.engine = engine

    // register the document
    this.engine.registerDocument(this.doc.UUID, this.doc)
    // TODO: to allow cross document references
    // we need to register a name, too
    // e.g. doc.UUID -> 'doc-1'
    this.engine.registerScope('doc', this.doc.UUID)

    // register all existing cells
    this._initialize()

    this.editorSession.on('render', this._onDocumentChange, this, {
      resource: 'document'
    })
  }

  _initialize() {
    forEach(this.doc.getNodes(), (node) => {
      this._onCreate(node)
    })
  }

  /*
    Update the engine when:
    - a cell/input is created
    - a cell/input is deleted
    - cell source code is changed
    - input name is changed
    - input value is changed
  */
  _onDocumentChange(change) {
    const doc = this.doc
    const ops = change.ops
    // let needsUpdate = false
    for (let i = 0; i < ops.length; i++) {
      const op = ops[i]
      switch (op.type) {
        case 'create': {
          let node = doc.get(op.path[0])
          if (node && this._onCreate(node)) {
            // console.log('detected cell creation')
            // needsUpdate = true
          }
          break
        }
        case 'delete': {
          if (this._onDelete(op.val)) {
            // console.log('detected cell deletion')
            // needsUpdate = true
          }
          break
        }
        case 'set':
        case 'update': {
          let node = doc.get(op.path[0])
          if (this._onChange(node, op)) {
            // console.log('detected cell update')
            // needsUpdate = true
          }
          break
        }
        default:
          //
      }
    }
    // TODO: let's see if we still gonna need this
    // if (needsUpdate) engine.update()
  }

  _onCreate(node) {
    const engine = this.engine
    if (CELL_TYPES[node.type]) {
      let adapter = new CellAdapter(node)
      engine.addCell(adapter)
      return true
    } else if (INPUT_TYPES[node.type]) {
      let adapter = new InputAdapter(node)
      engine.addCell(adapter)
      return true
    }
    return false
  }

  _onDelete(node) {
    const engine = this.engine
    if (CELL_TYPES[node.type] || INPUT_TYPES[node.type]) {
      engine.removeCell(node.id)
      return true
    }
    return false
  }

  _onChange(node) {
    const engine = this.engine
    if (node.type === 'source-code') {
      let cell = node.parentNode
      engine.updateCell(cell.id)
      return true
    } else if (INPUT_TYPES[node.type]) {
      // TODO: this needs to be rethought
      // const propName = op.path[1]
      // if (propName === 'value') {
      //   engine.updateInputValue(node.id)
      //   return true
      // }
      // // ATTENTION: input name should only be changed via SET operation
      // else if (propName === 'name') {
      //   engine.updateInputName(node.id, op.original)
      //   return true
      // }
    }
    return false
  }

}

class NodeAdapter {
  constructor(node) {
    this.node = node
  }

  emit(...args) {
    this.node.emit(...args)
  }

  isCell() {
    return false
  }

  isInput() {
    return false
  }

  get id() {
    return this.node.id
  }

  get docId() {
    return this.node.document.UUID
  }

}

class CellAdapter extends NodeAdapter {

  constructor(cell) {
    super(cell)
    this.node = cell

    this.state = new CellState()
    cell.state = this.state
  }

  isCell() {
    return true
  }

  get source() {
    const sourceEl = this._getSourceElement()
    return sourceEl.textContent
  }

  get language() {
    const sourceEl = this._getSourceElement()
    return sourceEl.getAttribute('language')
  }

  get inputs() {
    return this.state.inputs
  }

  get output() {
    return this.state.output
  }

  get value() {
    return this.state.value
  }

  _getSourceElement() {
    if (!this._source) {
      this._source = this.node.find('source-code')
    }
    return this._source
  }

}

class InputAdapter extends NodeAdapter {

  constructor(input) {
    super(input)
  }

  isInput() {
    return true
  }

  get name() {
    return this.node.getAttribute('name')
  }

  get value() {
    return this.node.getAttribute('value')
  }
}