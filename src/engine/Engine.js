import { uuid, isString, EventEmitter } from 'substance'
import CellGraph from './CellGraph'
import { ContextError, RuntimeError } from './CellErrors'
import { READY } from './CellStates'
import Cell from './Cell'
import CellSymbol from './CellSymbol'
import { parseSymbol } from '../shared/expressionHelpers'
import { getRowCol, valueFromText, getCellLabel } from '../shared/cellHelpers'

/*
  WIP
  The Engine implements the Stencila Execution Model.

  As the Engine can be run independently, and thus has its own model.
  There are two types of resources containing cells, Documents and Sheets.
  Every document defines a variable scope. Variables are produced by cells.
  A document has an id but also a human readable name.
  Sheets have a tabular layout, while Documents have a sequential layout.

  Document cells can define variables which can be referenced within the same document,
  like in `x + 1`.

  Sheet cells can be referenced via cell- and range expressions, such as
  `A1`, or `A1:B10`.

  Across documents and sheets, cells are referenced using a transclusion syntax, prefixed with the document id or the document name, such as in
  `'My Document'!x` or `sheet1!A1:B10`.

  > TODO: ATM we do not support other type of sheet references, such as via column name
  > or defining custom ranges.

  > Idea: use the column name as a reference to the corresponding cell in the same row
  > I.e. instead of `= A1 * B1` write `= width * height`

  Engine API
  - register document (type, id)
  - update document/sheet meta data (name, column information)
  - add cell
  - remove cell
  - set breakpoint / pause a cell
  - update cell data
  - update cell order (documents)
  - insert rows/cols
  - remove rows/cols

  While most of the time it is enough to look at cells independent of their
  documnents topologoy, this is necessary for Sheets in general, and for
  document cells with side-effects (global variables)

  Sheet specifics:
  - columns have meta data (name and type)
  - cell type comes either from cell data, or from its column (necessary for type validation)
  - spanning cells are rather a visual aspect. E.g. in GSheets the app
    clears spanned cells, and thus cell references yield empty values

  Open Questions:

  Should the Engine be run inside the Application/Render thread?

  On the one hand this could help to lower the load on the rendering thread.
  On the other hand, it is very usefule to have a more direct linking
  between the application and the engine: e.g. sharing the Host instance,
  and in the other direction.
  It is more important to run all contexts independently, so that
  code can be executed in multiple threads.

  How should we address merged/spanning cells?


  How do structural changes of sheets affect the cell graph?

  Sheet cells produce variables that look like `sheet1!A1`.
  Changing the structure of a sheet means that all cells after
  that need to be re-assigned. Changing the output symbol name only should not lead to a re-evaluation
  of the cell.
  The current state propagation mechanism does probably lead to potentially
  unnecessary re-evaluations when structure has been changed.
  This is because any kind of structural change leads to a reset of cell state
  We should improve this at some point. For now, it is not
  critical, because structural changes in sheets do not happen often,
  and in documents re-evaluation is most often necessary anyways.

*/
export default class Engine extends EventEmitter {

  constructor(host) {
    super()

    this._host = host

    this._docs = {}
    this._graph = new CellGraph()

    // for every (actionable) cell there is information what to do next
    // There are several steps that need to be done, to complete a cell:
    // - code analysis (context)
    // - registration of inputs/output (graph)
    // - cell evaluation (context)
    // - validation (engine)
    // - graph update
    this._nextActions = new Map()
  }

  run(interval) {
    // TODO: does this only work in the browser?
    if (this._runner) {
      clearInterval(this._runner)
    }
    this._runner = setInterval(() => {
      if (this.needsUpdate()) {
        this.cycle()
      }
    }, interval)
  }

  /*
    Registers a document via id.

    @param {object} data
      - `type`: 'document' | 'sheet'
      - `name`: a human readable name used for transclusions
      - `columns`: (for sheets) initial column data
      - 'sequence': (for documents) initial order of cells
  */
  addDocument(data) {
    let doc = new Document(data)
    this._registerResource(doc)
    return doc
  }

  addSheet(data) {
    let sheet = new Sheet(data)
    this._registerResource(sheet)
    return sheet
  }

  needsUpdate() {
    return this._nextActions.size > 0 || this._graph.needsUpdate()
  }

  cycle() {
    const graph = this._graph
    const nextActions = this._nextActions
    if (nextActions.size > 0) {
      // clearing next actions so that we can record new next actions
      this._nextActions = new Map()

      // group actions by type
      let actions = {
        analyse: [],
        register: [],
        evaluate: [],
        update: []
      }
      nextActions.forEach(a => actions[a.type].push(a))
      actions.update.forEach(a => {
        if (a.errors && a.errors.length > 0) {
          graph.addErrors(a.id, a.errors)
        } else {
          graph.setValue(a.id, a.value)
        }
      })
      actions.register.forEach(a => {
        let cell = graph.getCell(a.id)
        if (cell.isSheetCell()) {
          graph.setInputs(cell.id, a.inputs)
        } else {
          graph.setInputsOutputs(cell.id, a.inputs, a.output)
        }
      })

      this._updateGraph()

      let A = actions.analyse.map(a => this._analyse(a))
      let B = actions.evaluate.map(a => this._evaluate(a))
      return A.concat(B)
    } else if (graph.needsUpdate()) {
      this._updateGraph()
      return Promise.resolve(true)
    } else {
      return Promise.resolve(false)
    }
  }

  getNextActions() {
    return this._nextActions
  }

  _registerResource(doc) {
    const id = doc.id
    if (this._docs.hasOwnProperty(id)) throw new Error(`document with id ${id} already exists`)
    this._docs[id] = doc
    doc._registerCells(this)
  }


  /*
    Registers a cell.

    A cell is registered independent from the topology it resides in.

    Cells are treated differently w.r.t. their parent document.

    For instance, in a document cells can be block expressions,
    and can define a variable. In a sheet every cell must be a simple expression
    and it is is assigned to a variable implicitly (such as `sheet1!A1`).
  */
  _registerCell(cell) {
    this._graph.addCell(cell)
    this._updateCell(cell.id, cell)
  }

  /*
    Removes a cell from the engine.
  */
  _removeCell(id) { // eslint-disable-line
    // TODO
  }

  _updateCell(id, cellData) {
    // TODO: instead of waiting for another cycle
    // we could update the CellGraph right away
    // if in case of sheet cells the source is not an expression
    this._nextActions.set(id, {
      id,
      type: 'analyse',
      cellData,
      // used to detect invalidations
      token: uuid(),
    })
  }

  _sendUpdate(updatedCells) {
    // TODO: this should send a batch update over to the app
    // and for testing this method should be 'spied'
    this.emit('update', updatedCells)
  }

  _updateGraph() {
    const graph = this._graph
    let updatedIds = graph.update()
    let updatedCells = []
    updatedIds.forEach(id => {
      let cell = graph.getCell(id)
      if (cell) {
        if (cell.status === READY) {
          this._nextActions.set(cell.id, {
            type: 'evaluate',
            id: cell.id
          })
        }
        updatedCells.push(cell)
      }
    })
    if (updatedCells.length > 0) {
      this._sendUpdate(updatedCells)
    }
  }

  _analyse(action) {
    const graph = this._graph
    const id = action.id
    const cell = graph.getCell(id)
    // in case of constants, casting the string into a value,
    // updating the cell graph and returning without further evaluation
    if (cell.isConstant()) {
      // TODO: use the preferred type from the sheet
      let preferredType = 'any'
      let value = valueFromText(preferredType, cell.source)
      graph.setValue(id, value)
      return
    }
    // otherwise the cell source is assumed to be dynamic source code
    const transpiledSource = cell.transpiledSource
    const lang = cell.getLang()
    return this._getContext(lang)
    .then(res => {
      if (res instanceof Error) {
        const msg = `Could not get context for ${lang}`
        console.error(msg)
        let err = new ContextError(msg, { lang })
        graph.addError(id, err)
      } else {
        const context = res
        return context.analyseCode(transpiledSource)
      }
    })
    .then(res => {
      if (!res) return
      // console.log('ANALYSED cell', cell, res)
      // transform the extracted symbols into fully-qualified symbols
      // e.g. in `x` in `sheet1` is compiled into `sheet1.x`
      let { inputs, output } = this._compile(res, cell)
      this._nextActions.set(id, {
        type: 'register',
        id,
        inputs,
        output
      })
    })
  }

  _evaluate(action) {
    const graph = this._graph
    const id = action.id
    const cell = graph.getCell(id)
    console.log('evaluating cell', cell.toString())
    const lang = cell.getLang()
    let transpiledSource = cell.transpiledSource
    return this._getContext(lang)
    .then(res => {
      if (res instanceof Error) {
        const msg = `Could not get context for ${lang}`
        console.error(msg)
        let err = new ContextError(msg, { lang })
        graph.addError(id, err)
      } else {
        const context = res
        // console.log('EXECUTING cell', cell.id, transpiledSource)
        let inputs = this._getInputValues(cell.inputs)
        return context.executeCode(transpiledSource, inputs)
      }
    })
    .then(res => {
      this._nextActions.set(id, {
        type: 'update',
        id,
        errors: res.messages,
        value: res.value
      })
    })
  }

  _compile(res, cell) {
    const scopeId = cell.docId
    const symbolMapping = cell.symbolMapping
    let inputs = new Set()
    res.inputs.forEach(str => {
      // TODO: we already have transpiled the symbol, can we reuse it here?
      str = symbolMapping[str] || str
      const { type, scope, name, mangledStr } = parseSymbol(str)
      // if there is a scope given explicily try to lookup the doc
      let _scopeId = scopeId
      if (scope) {
        // Note: a failed lookup will eventually lead to a broken dependency
        // thus, we rely on the CellGraph to figure this out
        _scopeId = this._lookupDocumentId(scope) || scope
      }
      let qualifiedId = _scopeId + '!' + name
      const symbol = new CellSymbol(type, qualifiedId, _scopeId, name, mangledStr)
      inputs.add(symbol)
    })
    // turn the output into a qualified id
    let output
    if (res.output) output = scopeId + '!' + res.output
    return { inputs, output }
  }

  /*
    Provides packed values stored in a hash by their name.
    Ranges and transcluded symbols are stored via their mangled name.

    > Attention: this requires that cell code is being transpiled accordingly.

    ```
    $ graph._getInputValues(['x', 'sheet1!A1:B3'])
    {
      'x': ...,
      'sheet1_A1_B3': ...
    }
    ```
  */
  _getInputValues(inputs) {
    const graph = this._graph
    let result = {}
    inputs.forEach(symbol => {
      // Note: the transpiled source code is used for evaluation
      // thus we expose values via transpiled/mangled names here
      let val = graph.getValue(symbol)
      result[symbol.mangledStr] = val
    })
    return result
  }

  _getContext(lang) {
    return this._host.createContext(lang)
  }

  _createRuntimeErrors(messages) {
    if (messages) {
      return messages.map(msg => {
        return new RuntimeError(msg)
      })
    } else {
      return []
    }
  }

  _lookupDocumentId(name) {
    for (var id in this.docs) { // eslint-disable-line guard-for-in
      let doc = this.docs
      if (doc.name === name || id === name) {
        return doc.id
      }
    }
  }

}

/*
  Engine's Internal model of a Document.

  WIP: Aim is create a simple model for all types of
  documents/notebooks, independent from the document model used by Stencila.
*/
class Document {

  constructor(data) {
    const docId = data.id
    if (!docId) throw new Error("'id' is required")
    this.id = docId
    this.name = data.name
    // default language
    const defaultLang = data.lang || 'mini'
    this.lang = defaultLang

    this.cells = data.cells.map(cellData => {
      if (isString(cellData)) {
        let source = cellData
        cellData = {
          id: uuid(),
          docId,
          source
        }
      }
      let cell = new Cell(this, cellData)
      return cell
    })
  }

  get type() { return 'document' }

  getCells() {
    return this.cells
  }

  _registerCells(engine) {
    this.cells.forEach(cell => engine._registerCell(cell))
  }

  _updateDocumentSequence(docId, cellIds) { // eslint-disable-line
    // update the graph accordingly
  }

}

/*
  Engine's Internal model of a Spreadsheet.

  WIP: Aim is create a simple model for all types of
  spreadsheets, independent from the document model used by Stencila.
*/
class Sheet {

  constructor(data) {
    const docId = data.id
    if (!docId) throw new Error("'id' is required")
    this.id = docId
    // default language
    const defaultLang = data.lang || 'mini'
    this.lang = defaultLang
    // TODO: we can revise this as we move on
    // for now, data.cells must be present being a sequence of rows of cells.
    // data.columns is optional, but if present every data row have corresponding dimensions
    if (!data.cells) throw new Error("'cells' is mandatory")
    if (data.columns) {
      this.columns = data.columns
    } else {
      let ncols = data.cells[0].length
      this.columns = new Array(ncols).map(() => {
        return { type: 'auto' }
      })
    }
    const ncols = this.columns.length
    this.cells = data.cells.map((rowData, rowIdx) => {
      if (rowData.length !== ncols) throw new Error('Invalid data')
      let row = rowData.map((cellData, colIdx) => {
        // simple format: just the expression
        if (isString(cellData)) {
          let source = cellData
          cellData = {
            id: uuid(),
            docId,
            source,
            output: docId + '!' + getCellLabel(rowIdx, colIdx)
          }
        }
        let cell = new Cell(this, cellData)
        return cell
      })
      return row
    })
  }

  get type() { return 'sheet' }

  getCells(query) {
    if (!query) {
      return this.cells
    } else {
      let { type, name } = parseSymbol(query)
      switch (type) {
        case 'cell': {
          const [row, col] = getRowCol(name)
          return this.cells[row][col]
        }
        case 'range': {
          let [anchor, focus] = name.split(':')
          const [anchorRow, anchorCol] = getRowCol(anchor)
          const [focusRow, focusCol] = getRowCol(focus)
          if (anchorRow === focusRow && anchorCol === focusCol) {
            return this.cells[anchorCol][focusCol]
          }
          if (anchorRow === focusRow) {
            return this.cells[anchorRow].slice(anchorCol, focusCol+1)
          }
          if (anchorCol === focusCol) {
            let cells = []
            for (let i = anchorRow; i <= focusRow; i++) {
              cells.push(this.cells[i][anchorCol])
            }
            return cells
          }
          throw new Error('Unsupported query')
        }
        default:
          throw new Error('Unsupported query')
      }
      // single cell
      // or cell array
      // or cell matrix
    }
  }

  _registerCells(engine) {
    this.cells.forEach(row => row.forEach(cell => engine._registerCell(cell)))
  }

  /*
    Registers a sheet column.
  */
  _addColumn(sheetId, idx, colData, cellIds) { // eslint-disable-line
    // TODO:
    // - make sure that cells have been registered already
    // - length of cellIds must be consistent with sheet dimensions
    // - create a column record
  }

  _removeColumn(sheetId, idx) { // eslint-disable-line
    // TODO:
    // - remove the column record
    // - and update sheet symbols
  }

  _updateColumn(sheetId, idx, colData) { // eslint-disable-line
    // TODO:
    // - update column meta data
    // - invalidate cells accordingly (e.g. for type validation)
  }

  _addRow(sheetId, rowIdx, cellIds) { // eslint-disable-line
    // TODO:
    // - make sure that cells have been registered already
    // - length of cellIds must be consistent with sheet dimensions
    // - update column records
  }

  _removeRow(sheetId, rowIdx) { // eslint-disable-line
    // - update column records
  }

  _updateSheetSymbols(sheetId) { // eslint-disable-line
    // TODO:
    // - set output symbols of all sheet cells
    // - according to the current sheet structure

    // maybe this could be done automatically, however, this API
    // is considered low-level and I want to avoid that this is called
    // unnecessarily often
  }

}
