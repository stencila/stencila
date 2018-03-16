import { DocumentAdapter } from '../shared/DocumentAdapter'

/*
  Connects Engine and Sheet.
*/
export default class SheetAdapter extends DocumentAdapter {

  _initialize() {
    // find all
    const docId = this._getDocId()
    const sheet = this.doc
    const engine = this.engine
    // create a matrix of cells from the model
    // TODO: maybe the SheetDocument provide this as a random
    // access layer on top of the model
    let matrix = sheet.getCellMatrix()
    let nodeAdapters = matrix.map(row => {
      return row.map(node => this.adaptNode(node))
    })
    // TODO: also provide column data
    let sheetAdapter = engine.addSheet({
      id: docId,
      name: this.name,
      // default language
      lang: 'mini',
      cells: nodeAdapters
    })
    // use the Engine's cell data as state of the Article's node
    // NOTE: we can do this as long as we run the Engine in the same thread
    // Otherwise we would need to keep a local version of the state
    let cells = sheetAdapter.getCells()
    for (let i = 0; i < cells.length; i++) {
      let row = cells[i]
      for (let j = 0; j < row.length; j++) {
        const cell = row[j]
        nodeAdapters[i][j].node.state = cell
      }
    }
    this.editorSession.on('render', this._onDocumentChange, this, { resource: 'document' })
    this.engine.on('update', this._onEngineUpdate, this)
  }

  _onDocumentChange(change) { // eslint-disable-line
  }

  _onCreate(node) { // eslint-disable-line
  }

  _onDelete(node) { // eslint-disable-line
  }

  _onChange(node) { // eslint-disable-line
  }

  static connect(engine, editorSession, name) {
    return new SheetAdapter(engine, editorSession, name)
  }
}
