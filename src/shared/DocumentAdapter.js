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
    this.name = name
    // Note: this id is used internally to
    // lookup variables and cells
    if (!this.doc.UUID) {
      this.doc.UUID = uuid()
    }
    this._initialize()
  }

  _initialize() {
    throw new Error('This method is abstract')
  }

  _getDocId() {
    return this.doc.UUID
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

  adaptNode(node) {
    if (node._engineAdapter) return node._engineAdapter
    return this._adaptNode(node)
  }

  _adaptNode(node) {
    return new CellAdapter(node)
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
