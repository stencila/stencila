import { uuid, DocumentChange } from 'substance'
import { qualifiedId } from './cellHelpers'

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
    // WORKAROND: because Substance Document does not provide a setter
    // for `id` we are setting the underlying property directly
    if (!this.doc.id) this.doc.__id__ = uuid()
    this.name = name

    this._initialize()
  }

  _initialize() {
    throw new Error('This method is abstract')
  }

  /*
    Called after each Engine cycle.

    @param {Set<Cell>} updates updated cells.
  */
  _onEngineUpdate(updates) {
    // Note: for now we are sharing the cell states with the engine
    // thus, we can just notify the session about the changed cells
    const docId = this.doc.id
    const editorSession = this.editorSession
    let nodeIds = updates.filter(cell => cell.docId === docId).map(cell => cell.unqualifiedId)
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


export function getQualifiedId(node) {
  if (!node._qualifiedId) {
    node._qualifiedId = qualifiedId(node.getDocument(), node)
  }
  return node._qualifiedId
}

export function mapCellState(doc, cellState) {
  // TODO: we need to be careful with this
  // The node state should be something general, document specific
  // Instead we take all necessary parts of the engine's cell state
  // and use the document's node state API (future)
  // For now, we just share the state
  let node = doc.get(cellState.unqualifiedId)
  node.state = cellState
}