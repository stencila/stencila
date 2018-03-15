import { uuid } from 'substance'

/*
  Connects Engine and Sheet.
*/
export default class SheetAdapter {

  constructor(engine, editorSession, name) {
    this.engine = engine
    this.editorSession = editorSession
    this.doc = editorSession.getDocument()
    this.name = name

    // Note: this id is used internally to
    // lookup variables and cells
    if (!this.doc.UUID) {
      this.doc.UUID = uuid()
    }
  }

  _initialize() {
    // forEach(this.doc.getNodes(), (node) => {
    //   this._onCreate(node)
    // })
    // this.editorSession.on('render', this._onDocumentChange, this, {
    //   resource: 'document'
    // })
  }

  _onDocumentChange(change) {
    // const doc = this.doc
    // const ops = change.ops
    // for (let i = 0; i < ops.length; i++) {
    //   const op = ops[i]
    //   switch (op.type) {
    //     case 'create': {
    //       let node = doc.get(op.path[0])
    //       if (node) {
    //         this._onCreate(node)
    //       }
    //       break
    //     }
    //     case 'delete': {
    //       this._onDelete(op.val)
    //       break
    //     }
    //     case 'set':
    //     case 'update': {
    //       let node = doc.get(op.path[0])
    //       this._onChange(node, op)
    //       break
    //     }
    //     default:
    //       //
    //   }
    // }
  }

  _onCreate(node) {
    // const engine = this.engine
    // if (CELL_TYPES[node.type]) {
    //   let adapter = new CellAdapter(this.editorSession, node)
    //   if (adapter._type === EXPRESSION) {
    //     engine.registerCell(adapter)
    //   }
    // }
  }

  _onDelete(node) {
    // const engine = this.engine
    // if (CELL_TYPES[node.type]) {
    //   engine.removeCell(`${this.doc.UUID}#${node.id}`)
    // }
  }

  _onChange(node) {
    // const engine = this.engine
    // if (CELL_TYPES[node.type]) {
    //   let adapter = node._adapter
    //   let oldType = adapter._type
    //   adapter._update()
    //   let newType = adapter._type
    //   if (newType === CONSTANT) {
    //     if (oldType === EXPRESSION) {
    //       engine.removeCell(adapter.id)
    //     } else {
    //       engine.updateInput(adapter.id)
    //     }
    //   } else {
    //     if (oldType === CONSTANT) {
    //       engine.registerCell(adapter)
    //     } else {
    //       engine.updateCell(adapter.id)
    //     }
    //   }
    // }
  }

  static connect(engine, editorSession, name) {
    return new SheetAdapter(engine, editorSession, name)
  }
}

class CellAdapter {

  constructor(node) {
    this.node = node
    // store this adapter so that we can reuse it later
    node._engineAdapter = this
  }

  get id() {
    return this.node.id
  }

  get source() {
    return this.node.textContent
  }

  get lang() {
    return this.node.getAttribute('language')
  }

}

function adaptNode(node) {
  if (node._engineAdapter) return node._engineAdapter
  return new CellAdapter(node)
}