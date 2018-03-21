import { uuid, DocumentChange } from 'substance'

/*
  Base-Class for adapters between document and engine.
*/
export class DocumentAdapter {

  // TODO: dicuss ownership of 'name'
  // It seems that this is a property inside the manifest,
  // and not part of the JATS file
  constructor(engine, editorSession, name) {
    this.engine = engine
    this.editorSession = editorSession
    this.doc = editorSession.getDocument()
    // a unique id to identify documents (e.g. used as for variable scopes and transclusions)
    if (!this.doc.UUID) this.doc.UUID = uuid()
    this.docId = this.doc.UUID
    this.name = name

    // used internally to record changes applied to the document model
    // and that need to transfered to the Engine.
    this._updates = new Map()

    this._initialize()
  }

  _initialize() {
    throw new Error('This method is abstract')
  }

  /*
    Called after every DocumentChange to keep the Engine in sync.
  */
  _update() {
    throw new Error('This method is abstract.')
  }

  _adaptNode(node) {
    if (node._engineAdapter) return node._engineAdapter
    return new CellAdapter(node)
  }

  _qualifiedId(cell) {
    // TODO: where is this notation coming from?
    return `${this.docId}_${cell.id}`
  }

  /*
    Called after each Engine cycle.

    @param {Set<Cell>} updates updated cells.
  */
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
}

export class CellAdapter {

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
    return this.node
  }

}
