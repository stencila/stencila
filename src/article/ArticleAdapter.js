import { DocumentAdapter, CellAdapter } from '../shared/DocumentAdapter'

/*
  Connects Engine and Article.
*/
export default class ArticleAdapter extends DocumentAdapter {

  _initialize() {
    const docId = this.docId
    const doc = this.doc
    const engine = this.engine
    // a plain
    let adapter = engine.addDocument({
      id: docId,
      name: this.name,
      lang: 'mini',
      cells: this._getNodeAdapters(),
      onRegister(cell) {
        let node = doc.get(cell.localId)
        node.state = cell
      }
    })
    this.adapter = adapter
    this.editorSession.on('render', this._onDocumentChange, this, { resource: 'document' })
    this.engine.on('update', this._onEngineUpdate, this)
  }

  _getNodeAdapters() {
    // TODO: in future there might be more node types that need to be registered
    return this.doc.findAll('cell').map(node => this._adaptNode(node))
  }

  /*
    Call on every document change detecting updates to cells that
    are used to keep the Engine's model in sync.
  */
  _onDocumentChange(change) {
    const doc = this.doc
    const adapter = this.adapter
    const engine = this.engine

    const ops = change.ops
    let updates = new Map()
    for (let i = 0; i < ops.length; i++) {
      const op = ops[i]
      switch (op.type) {
        case 'create': {
          let node = doc.get(op.path[0])
          if (node && this._isCell(node)) {
            updates.set(node.id, { type: 'add', node })
          }
          break
        }
        case 'delete': {
          // TODO: would be good to still have the node instance
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
          if (node) {
            if (node.type === 'source-code') {
              let cellNode = node.parentNode
              this._updates.set(cellNode.id, { type: 'change', node: cellNode })
            }
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
    // TODO: updating the array of cells feels a bit clumsy,
    // still we want to go down this route to make the Engine more independent of
    // specific application models (such as Stencila Document/Sheet).
    if (structureChanged) {
      adapter.cells = this._getNodeAdapters()
    }
  }


  _adaptNode(node) {
    if (node._engineAdapter) return node._engineAdapter
    return new ArticleCellAdapter(node)
  }

  /*
    Used internally to filter cells.
  */
  _isCell(node) {
    return node.type === 'cell'
  }


  static connect(engine, editorSession, name) {
    return new ArticleAdapter(engine, editorSession, name)
  }
}

/*
  An adpater for cells from Stencila Articles.
*/
class ArticleCellAdapter extends CellAdapter {

  /*
    Article cells have a special layout, with the source code in an extra element.
  */
  _getSourceElement() {
    if (!this._sourceEl) {
      this._sourceEl = this.node.find('source-code')
    }
    return this._sourceEl
  }
}
