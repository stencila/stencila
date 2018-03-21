import { DocumentAdapter } from '../shared/DocumentAdapter'

/*
  Connects Engine and Sheet.
*/
export default class SheetAdapter extends DocumentAdapter {

  _initialize() {
    const doc = this.doc
    const docId = this.docId
    const engine = this.engine
    // TODO: also provide column data
    this.adapter = engine.addSheet({
      id: docId,
      name: this.name,
      // default language
      lang: 'mini',
      cells: this._getNodeAdapters(),
      onRegister(cell) {
        let node = doc.get(cell.localId)
        node.state = cell
      }
    })
    this.editorSession.on('render', this._onDocumentChange, this, { resource: 'document' })
    this.engine.on('update', this._onEngineUpdate, this)
  }

  _onDocumentChange(change) {
    // TODO: deal with updates to columns (name, types)

    // TODO: with 'semantic' changes this could be much simpler,
    // i.e. `change.info.action` consistently, and then detect
    // structural changes more easily.
    // For now we follow the old approach handling changes on op-per-op basis
    const doc = this.doc
    const adapter = this.adapter
    const engine = this.engine
    const ops = change.ops
    let updates = new Map()
    for (let i = 0; i < ops.length; i++) {
      const op = ops[i]
      // TODO: this very similar to what is done in ArticleAdapter
      // we should reuse code, by either moving things into DocumentAdapter
      // or by using helpers
      switch (op.type) {
        case 'create': {
          let node = doc.get(op.path[0])
          if (node && this._isCell(node)) {
            updates.set(node.id, { type: 'add', node })
          }
          break
        }
        case 'delete': {
          let nodeData = op.val
          if (this._isCell(nodeData)) {
            updates.set(nodeData.id, { type: 'remove', node: nodeData })
          }
          break
        }
        case 'set':
        case 'update': {
          let node = doc.get(op.path[0])
          // node can be null if it has been deleted later on
          if (this._isCell(node)) {
            this._updates.set(node.id, { type: 'change', node })
          }
          break
        }
        default:
          throw new Error('Invalid state')
      }
    }
    // stop here if there were no updates
    if (updates.size === 0) return
    // otherwise update the engine
    // TODO: we need to consolidate the adapter/engine API
    // either we use only the Engine's document API
    // or only use the Engine's API
    let structureChanged = false
    updates.forEach(update => {
      const node = update.node
      switch (update.type) {
        case 'add': {
          adapter.registerCell(this._adaptNode(node))
          structureChanged = true
          break
        }
        case 'remove': {
          engine.removeCell(this._qualifiedId(node))
          structureChanged = true
          break
        }
        case 'change': {
          // TODO: we should use a helper to access the hidden fields 'state' and '_engineAdapter'
          let cellState = node.state
          let cellAdapter = node._engineAdapter
          if (cellState && cellAdapter) {
            const { source, lang } = cellAdapter
            engine._updateCell(cellState.id, { source, lang })
          }
          break
        }
        default:
          throw new Error('Invalid update.')
      }
    })
    if (structureChanged) {
      // TODO: updating the matrix feels still a bit clumsy here
      adapter.cells = this._getNodeAdapters()
      adapter.updateCellSymbols(engine)
    }
  }


  _getNodeAdapters() {
    let matrix = this.doc.getCellMatrix()
    return matrix.map(row => {
      return row.map(node => this._adaptNode(node))
    })
  }

  _isCell(node) {
    return node.type === 'cell'
  }

  static connect(engine, editorSession, name) {
    return new SheetAdapter(engine, editorSession, name)
  }
}
