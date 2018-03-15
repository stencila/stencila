import { uuid, DocumentChange } from 'substance'

/*
  Connects Engine and Article.
*/
export default class ArticleAdapter {

  // TODO: dicuss ownership of 'name'
  // It seems that this is a property inside the manifest,
  // and not part of the JATS file
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
    this._initialize()
  }

  _initialize() {
    // find all
    const docId = this.doc.UUID
    let nodeAdapters = this.doc.findAll('cell').map(node => adaptNode(node))
    let docAdapter = this.engine.addDocument({
      id: docId,
      name: this.name,
      // default language
      lang: 'mini',
      cells: nodeAdapters
    })
    // use the Engine's cell data as state of the Article's node
    // NOTE: as long as we do run the Engine in the same thread
    // as the model, we can do this. Otherwise we would need to
    // keep a local version of the state
    docAdapter.getCells().forEach((cell,idx) => {
      nodeAdapters[idx].node.state = cell
    })
    this.editorSession.on('render', this._onDocumentChange, this, {
      resource: 'document'
    })
    this.engine.on('update', this._onEngineUpdate, this)
  }

  _onEngineUpdate(updates) {
    // Note: for now we are sharing the cell states with the engine
    // thus, we can just notify the session about the changed cells
    const docId = this.doc.UUID
    const editorSession = this.editorSession
    let nodeIds = updates.filter(cell => cell.docId === docId).map(cell => cell.localId)
    if (nodeIds.length > 0) {
      // TODO: there should be a built in means to trigger a reflow
      // after updates of node states
      editorSession._setDirty('document')
      let change = new DocumentChange([], {}, {})
      change._extractInformation()
      nodeIds.forEach(nodeId => {
        change.updated[nodeId] = true
      })
      // TODO: what is this for?
      change.updated['setState'] = nodeIds
      editorSession._change = change
      editorSession._info = {}
      editorSession.startFlow()
    }
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
    //       if (node) {
    //         this._onChange(node, op)
    //       }
    //       break
    //     }
    //     default:
    //       throw new Error('Invalid state')
    //   }
    // }
  }

  _onCreate(node) {
    // const engine = this.engine
    // if (CELL_TYPES[node.type]) {
    //   let adapter = new CellAdapter(this.editorSession, node)
    //   engine.registerCell(adapter)
    // } else if (INPUT_TYPES[node.type]) {
    //   let adapter = new InputAdapter(this.editorSession, node)
    //   engine.registerCell(adapter)
    //   return true
    // }
    // return false
  }

  _onDelete(node) {
    // const engine = this.engine
    // if (CELL_TYPES[node.type] || INPUT_TYPES[node.type]) {
    //   engine.removeCell(`${this.doc.UUID}#${node.id}`)
    //   return true
    // }
    // return false
  }

  _onChange(node) {
    // const engine = this.engine
    // if (node.type === 'source-code') {
    //   let cell = node.parentNode
    //   engine.updateCell(getFullyQualifiedNodeId(cell))
    //   return true
    // } else if (INPUT_TYPES[node.type]) {
    //   // TODO: this needs to be rethought
    //   // const propName = op.path[1]
    //   // if (propName === 'value') {
    //   //   engine.updateInputValue(node.id)
    //   //   return true
    //   // }
    //   // // ATTENTION: input name should only be changed via SET operation
    //   // else if (propName === 'name') {
    //   //   engine.updateInputName(node.id, op.original)
    //   //   return true
    //   // }
    // }
    // return false
  }

  static connect(engine, editorSession, name) {
    return new ArticleAdapter(engine, editorSession, name)
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
    return this._getSourceElement().textContent
  }

  get lang() {
    return this._getSourceElement().getAttribute('language')
  }

  _getSourceElement() {
    if (!this._sourceEl) {
      this._sourceEl = this.node.find('source-code')
    }
    return this._sourceEl
  }

}

function adaptNode(node) {
  if (node._engineAdapter) return node._engineAdapter
  return new CellAdapter(node)
}
