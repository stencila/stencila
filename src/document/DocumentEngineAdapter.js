import { forEach } from 'substance'
import { getCellAdapter, getInputAdapter } from './cellHelpers'

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

  /*
    Update the engine when:
    - a cell/input is created
    - a cell/input is deleted
    - cell source code is changed
    - input name is changed
    - input value is changed
  */
  _onDocumentChange(change) {
    const engine = this.engine
    const doc = this.doc
    const ops = change.ops
    let needsUpdate = false
    for (let i = 0; i < ops.length; i++) {
      const op = ops[i]
      switch (op.type) {
        case 'create': {
          let node = doc.get(op.path[0])
          if (node && this._onCreate(node)) {
            // console.log('detected cell creation')
            needsUpdate = true
          }
          break
        }
        case 'delete': {
          if (this._onDelete(op.val)) {
            // console.log('detected cell deletion')
            needsUpdate = true
          }
          break
        }
        case 'set':
        case 'update': {
          let node = doc.get(op.path[0])
          if (this._onChange(node, op)) {
            // console.log('detected cell update')
            needsUpdate = true
          }
          break
        }
        default:
          //
      }
    }
    if (needsUpdate) engine.update()
  }

  _onCreate(node) {
    const engine = this.engine
    if (CELL_TYPES[node.type]) {
      let adapter = getCellAdapter(node)
      engine._registerCell(adapter)
      return true
    } else if (INPUT_TYPES[node.type]) {
      let adapter = getInputAdapter(node)
      engine._registerCell(adapter)
      return true
    }
    return false
  }

  _onDelete(node) {
    const engine = this.engine
    if (CELL_TYPES[node.type] || INPUT_TYPES[node.type]) {
      engine._deregisterCell(node.id)
      return true
    }
    return false
  }

  _onChange(node, op) {
    const engine = this.engine
    if (node.type === 'source-code') {
      let cell = node.parentNode
      engine._updateCell(cell.id)
      return true
    } else if (INPUT_TYPES[node.type]) {
      const propName = op.path[1]
      if (propName === 'value') {
        engine._updateInputValue(node.id)
        return true
      }
      // ATTENTION: input name should only be changed via SET operation
      else if (propName === 'name') {
        engine._updateInputName(node.id, op.original)
        return true
      }
    }
    return false
  }

}
