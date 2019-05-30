import { DocumentAdapter, mapCellState } from '../shared/DocumentAdapter'
import { getSource, getLang } from '../shared/cellHelpers'

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
      columns: this._getColumnNodes().map(_getColumnData),
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
    const doc = this.doc
    const model = this.model

    let action = change.info.action
    let matrix, cellData
    switch(action) {
      case 'insertRows': {
        const { pos, count } = change.info
        matrix = doc.getCellMatrix()
        cellData = []
        for (let i = pos; i < pos + count; i++) {
          cellData.push(matrix[i].map(_getCellData))
        }
        model.insertRows(pos, cellData)
        break
      }
      case 'deleteRows': {
        const { pos, count } = change.info
        model.deleteRows(pos, count)
        break
      }
      case 'insertCols': {
        const { pos, count } = change.info
        matrix = doc.getCellMatrix()
        cellData = []
        const N = matrix.length
        for (let i = 0; i < N; i++) {
          cellData.push(matrix[i].slice(pos, pos+count).map(_getCellData))
        }
        model.insertCols(pos, cellData)
        break
      }
      case 'deleteCols': {
        const { pos, count } = change.info
        model.deleteCols(pos, count)
        break
      }
      default: {
        // Note: only detecting updates on operation level
        // structural changes (insert/remove row/column) are special type of changes
        // TODO: deal with updates to columns (name, types)
        let updated
        const ops = change.ops
        for (let i = 0; i < ops.length; i++) {
          const op = ops[i]
          switch (op.type) {
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
    }
  }

  _getCellNodes() {
    return this.doc.getCellMatrix()
  }

  _getColumnNodes() {
    return this.doc.findAll('columns > col')
  }

  _isCell(node) {
    return node.type === 'cell'
  }

  static connect(engine, editorSession, id, name) {
    return new SheetAdapter(engine, editorSession, id, name)
  }
}

function _getCellData(cell) {
  return {
    id: cell.id,
    lang: getLang(cell),
    source: getSource(cell)
  }
}

function _getColumnName(column) {
  return column.getAttribute('name')
}

function _getColumnType(column) {
  return column.getAttribute('type')
}

function _getColumnData(column) {
  return {
    name: _getColumnName(column),
    type: _getColumnType(column)
  }
}
