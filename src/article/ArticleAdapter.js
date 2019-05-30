import { DocumentAdapter, mapCellState } from '../shared/DocumentAdapter'
import { getSource, getLang } from '../shared/cellHelpers'

/*
  Connects Engine and Article.
*/
export default class ArticleAdapter extends DocumentAdapter {

  _initialize() {
    const doc = this.doc
    const engine = this.engine

    // hack: monkey patching the instance to register a setter that updates this adapter
    _addAutorunFeature(doc, this)

    let model = engine.addDocument({
      id: doc.id,
      name: this.name,
      lang: 'mini',
      cells: this._getCellNodes().map(_getCellData),
      autorun: doc.autorun,
      onCellRegister: mapCellState.bind(null, doc)
    })
    this.model = model

    // TODO: do this somewhere else
    doc.autorun = false

    this.editorSession.on('update', this._onDocumentChange, this, { resource: 'document' })
    this.engine.on('update', this._onEngineUpdate, this)
  }

  _getCellNodes() {
    return this.doc.findAll('cell')
  }

  /*
    Call on every document change detecting updates to cells that
    are used to keep the Engine's model in sync.
  */
  _onDocumentChange(change) {
    const doc = this.doc
    const model = this.model
    // inspecting ops to detect structural changes and updates
    // Cell removals are applied directly to the engine model
    // while insertions are applied at the end
    // 1. removes, 2. creates, 3. updates, 3. creates
    let created, updated
    const ops = change.ops
    for (let i = 0; i < ops.length; i++) {
      const op = ops[i]
      switch (op.type) {
        case 'create': {
          let node = doc.get(op.path[0])
          if (this._isCell(node)) {
            if (!created) created = new Set()
            created.add(node.id)
          }
          break
        }
        case 'delete': {
          // TODO: would be good to still have the node instance
          let nodeData = op.val
          if (this._isCell(nodeData)) {
            model.removeCell(nodeData.id)
          }
          break
        }
        case 'set':
        case 'update': {
          let node = doc.get(op.path[0])
          // null if node is deleted within the same change
          if (!node) continue
          if (node.type === 'source-code') {
            node = node.parentNode
          }
          if (this._isCell(node)) {
            if (!updated) updated = new Set()
            updated.add(node.id)
          }
          break
        }
        default:
          throw new Error('Invalid state')
      }
    }
    if (created) {
      let cellNodes = this._getCellNodes()
      for (let i = 0; i < cellNodes.length; i++) {
        const cellNode = cellNodes[i]
        if (created.has(cellNode.id)) {
          model.insertCellAt(i, _getCellData(cellNode))
        }
      }
    }
    if (updated) {
      updated.forEach(id => {
        const cell = this.doc.get(id)
        const cellData = {
          source: getSource(cell),
          lang: getLang(cell)
        }
        model.updateCell(id, cellData)
      })
    }
  }

  /*
    Used internally to filter cells.
  */
  _isCell(node) {
    return node.type === 'cell'
  }


  static connect(engine, editorSession, id, name) {
    return new ArticleAdapter(engine, editorSession, id, name)
  }
}

function _getCellData(cell) {
  return {
    id: cell.id,
    lang: getLang(cell),
    source: getSource(cell)
  }
}

function _addAutorunFeature(doc, adapter) {
  Object.defineProperty(doc, 'autorun', {
    set(val) {
      doc._autorun = val
      adapter.model.setAutorun(val)
    },
    get() {
      return doc._autorun
    }
  })
}
