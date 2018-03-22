import { DocumentAdapter, getQualifiedId, mapCellState } from '../shared/DocumentAdapter'

/*
  Connects Engine and Sheet.
*/
export default class SheetAdapter extends DocumentAdapter {

  _initialize() {
    const doc = this.doc
    const engine = this.engine
    // TODO: also provide column data
    let model = engine.addSheet({
      id: doc.id,
      name: this.name,
      lang: 'mini',
      cells: this._getCellNodes().map(row => {
        return row.map(_getCellData)
      }),
      onCellRegister: mapCellState.bind(null, doc)
    })
    this.model = model

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
    const model = this.model
    // inspecting ops to detect structural changes and updates
    // Cell removals are applied directly to the engine model
    // while insertions are applied at the end
    // 1. removes, 2. creates, 3. updates, 3. creates
    let updated
    const ops = change.ops
    for (let i = 0; i < ops.length; i++) {
      const op = ops[i]
      // TODO: detect insert/remove row/col
      // TODO: detect change of column types
      switch (op.type) {
        case 'create': {
          break
        }
        case 'delete': {
          break
        }
        case 'set':
        case 'update': {
          let node = doc.get(op.path[0])
          // null if node is deleted within the same change
          if (!node) continue
          if (this._isCell(node)) {
            if (!updated) updated = new Set()
            updated.add(node.id)
          }
          break
        }
        default:
          //
      }
    }
    // TODO: insert/delete row/col
    if (updated) {
      updated.forEach(id => {
        const cell = this.doc.get(id)
        const qualifiedId = getQualifiedId(cell)
        const cellData = {
          source: _getSource(cell),
          lang: _getLang(cell)
        }
        model.updateCell(qualifiedId, cellData)
      })
    }
  }


  _getCellNodes() {
    return this.doc.getCellMatrix()
  }

  _isCell(node) {
    return node.type === 'cell'
  }

  static connect(engine, editorSession, name) {
    return new SheetAdapter(engine, editorSession, name)
  }
}

function _getSource(node) {
  return node.textContent
}

function _getLang(node) {
  return node.getAttribute('language')
}

function _getCellData(cell) {
  return {
    id: getQualifiedId(cell),
    lang: _getLang(cell),
    source: _getSource(cell)
  }
}
