import { uuid, DocumentChange } from 'substance'
import { qualifiedId, setSource } from './cellHelpers'

/*
  Base-Class for adapters between document and engine.
*/
export class DocumentAdapter {

  // TODO: dicuss ownership of 'name'
  // It seems that this is a property inside the manifest,
  // and not part of the JATS file
  constructor(engine, editorSession, id, name) {
    this.engine = engine
    this.editorSession = editorSession
    this.doc = editorSession.getDocument()
    this.id = id || uuid()
    this.name = name
    // a unique id to identify documents (e.g. used as for variable scopes and transclusions)
    // WORKAROND: because Substance Document does not provide a setter
    // for `id` we are setting the underlying property directly
    if (!this.doc.id) this.doc.__id__ = this.id

    this._initialize()
  }

  _initialize() {
    throw new Error('This method is abstract')
  }

  /*
    Called after each Engine cycle.

    @param {Set<Cell>} updates updated cells.
  */
  _onEngineUpdate(type, cellsByDocId) {
    // Note: for now we are sharing the cell states with the engine
    // thus, we can just notify the session about the changed cells
    const docId = this.doc.id
    const editorSession = this.editorSession
    let cells = cellsByDocId[docId]
    if (!cells || cells.length === 0) return
    if (type === 'state') {
      let nodeIds = cells.map(cell => cell.unqualifiedId)
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
    } else if (type === 'source') {
      // TODO: this should be easier after our EditorSession / AppState refactor in Substance
      const _update = () => {
        // TODO: we are probably messing up the undo history
        // to fix this, we need to to some 'rebasing' of changes in the history
        // as if this change was one of a collaborator.
        editorSession.transaction(tx => {
          cells.forEach(cell => {
            let cellNode = tx.get(cell.unqualifiedId)
            if (cellNode) {
              setSource(cellNode, cell.source)
            }
          })
        }, { action: 'setCellSource', history: false })
      }
      if (editorSession._flowing) {
        editorSession.postpone(_update)
      } else {
        _update()
      }
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