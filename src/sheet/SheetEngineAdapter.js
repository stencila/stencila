import { forEach } from 'substance'
import CellState from '../engine/CellState'

const CELL_TYPES = {
  'cell': true
}

const EXPRESSION = Symbol('expression')
const CONSTANT = Symbol('constant')

export default class SheetEngineAdapter {

  constructor(editorSession) {
    this.editorSession = editorSession
    this.doc = editorSession.getDocument()

    // set by Engine
    this.engine = null
  }

  connect(engine) {
    if (this.engine) throw new Error('This resource is already connected to an engine.')
    this.engine = engine

    this._initializeEngine()

    this.editorSession.on('render', this._onDocumentChange, this, {
      resource: 'document'
    })
  }

  _initializeEngine() {
    forEach(this.doc.getNodes(), (node) => {
      this._onCreate(node)
    })
  }

  _onDocumentChange(change) {
    const doc = this.doc
    const ops = change.ops
    for (let i = 0; i < ops.length; i++) {
      const op = ops[i]
      switch (op.type) {
        case 'create': {
          let node = doc.get(op.path[0])
          if (node) {
            this._onCreate(node)
          }
          break
        }
        case 'delete': {
          this._onDelete(op.val)
          break
        }
        case 'set':
        case 'update': {
          let node = doc.get(op.path[0])
          this._onChange(node, op)
          break
        }
        default:
          //
      }
    }
  }

  _onCreate(node) {
    const engine = this.engine
    if (CELL_TYPES[node.type]) {
      let adapter = new CellAdapter(node)
      if (adapter.type === 'expression') {
        engine.addCell(adapter)
      }
    }
  }

  _onDelete(node) {
    const engine = this.engine
    if (CELL_TYPES[node.type]) {
      engine.removeCell(node.id)
    }
  }

  _onChange(node) {
    const engine = this.engine
    if (CELL_TYPES[node.type]) {
      let adapter = node._adapter
      let oldType = adapter._type
      adapter._update()
      let newType = adapter._type
      if (newType === CONSTANT) {
        if (oldType === EXPRESSION) {
          engine.removeCell(node.id)
        } else {
          console.warn('TODO: trigger propagation of value')
        }
      } else {
        if (oldType === CONSTANT) {
          engine.addCell(adapter)
        } else {
          engine.updateCell(node.id)
        }
      }
    }
  }

}

class CellAdapter {

  constructor(cell) {
    this.node = cell

    this.state = new CellState()
    cell.state = this.state
    cell._adapter = this

    this._update()
  }

  emit(...args) {
    this.node.emit(...args)
  }

  isCell() {
    return this._type === EXPRESSION
  }

  isInput() {
    return this._type === CONSTANT
  }

  get id() {
    return this.node.id
  }

  get source() {
    return this._source
  }

  get language() {
    return this._lang
  }

  get inputs() {
    return this.state.inputs
  }

  get output() {
    return this.state.output
  }

  _update() {
    let source = this.node.textContent
    let prefix = /^\s*=/.exec(source)
    if (prefix) {
      this._type = EXPRESSION
      this._source = new Array(prefix.length).fill(' ') + source.substring(prefix.length)
      this._lang = this.node.getAttribute('language') || 'mini'
    } else {
      this._type = CONSTANT
      this._source = source
      this._lang = null
    }
  }

}
