import { uuid, isString, EventEmitter, flatten } from 'substance'
import CellGraph from './CellGraph'
import { ContextError, RuntimeError, SyntaxError } from './CellErrors'
import { UNKNOWN, ANALYSED, READY } from './CellStates'
import Cell from './Cell'
import CellSymbol from './CellSymbol'
import { parseSymbol } from '../shared/expressionHelpers'
import { getRowCol, valueFromText, getCellLabel, getColumnLabel, qualifiedId as _qualifiedId, queryCells } from '../shared/cellHelpers'
import { gather } from '../value'

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

  Sheet Ranges:

  Sheet Ranges such as `sheet1!A1:B10` are treated as independent symbols produced by so called RangeCells.
  RangeCells act as a proxy cell for the underlying cells.
  E.g. `sheet1!A1:B10` is produced by a RangeCell with dependencies `['A1', ..., 'B10']`.
  In contrast to a regular cell, a RangeCell does not need an extra evaluation step. Instead its value
  can be derived synchronously, as soon as all dependencies are ready.

  One challenge is that such cells should be pruned automatically, when not used anymore.
  E.g., when the source of a cell is changed from `sheet1!A1:B10` to `sheet1!A1:B11`,
  the former hidden cell should be removed if there is no other cell left that depends on it.


  Open Questions:

  Should the Engine be run inside the Application/Render thread?

  On the one hand this could help to lower the load on the rendering thread.
  On the other hand, it is very usefule to have a more direct linking
  between the application and the engine: e.g. sharing the Host instance,
  and in the other direction.
  It is more important to run all contexts independently, so that
  code can be executed in multiple threads.

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

  Sheet: should we allow to use column names as alias?

  ATM, when using a 2D cell range, a table value is created
  using column names when present, otherwise using the classical
  column label (e.g. 'A'). This is somewhat inconsistent,
  as someone could write code that makes use of the default column
  labels, say `filter(data, 'A < 20')`, which breaks as soon that column
  gets an explicit name.
  If we wanted to still allow this, we would need some kind of an alias mechanism
  in the table type.


*/
export default class Engine extends EventEmitter {

  constructor(options = {}) {
    super()

    // needs to be connected to a host to be able to create contexts
    this._host = options.host

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

  setHost(host) {
    this._host = host
  }

  run /* istanbul ignore next */ (interval) {
    if (!this._host) throw new Error('Must call setHost() before starting the Engine')

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
    let doc = new Document(this, data)
    this._registerResource(doc)
    return doc
  }

  addSheet(data) {
    let sheet = new Sheet(this, data)
    this._registerResource(sheet)
    return sheet
  }

  needsUpdate() {
    return this._nextActions.size > 0 || this._graph.needsUpdate()
  }

  cycle() {
    let res = []
    const graph = this._graph
    const nextActions = this._nextActions
    if (nextActions.size > 0) {
      // console.log('executing cycle')
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
        a.inputs.forEach(symbol => {
          if (symbol.type === 'range') {
            let rangeCell = graph.getCell(symbol)
            if (!rangeCell) {
              // console.log('registering RangeCell for', symbol.id)
              rangeCell = new RangeCell(symbol)
              graph.addCell(rangeCell)
            }
            rangeCell.refs++
          }
        })
        let cell = graph.getCell(a.id)
        // Note: we do a ref-couting for RangeCells and remove it
        // when it is not referenced somewhere else
        cell.inputs.forEach(symbol => {
          if (symbol.type === 'range') {
            let rangeCell = graph.getCell(symbol)
            rangeCell.refs--
            if (rangeCell.refs === 0) {
              // console.log('removing RangeCell for', rangeCell.symbol.id)
              graph.removeCell(rangeCell.id)
            }
          }
        })

        if (cell.isSheetCell()) {
          graph.setInputs(cell.id, a.inputs)
        } else {
          graph.setInputsOutputs(cell.id, a.inputs, a.output)
        }
      })

      this._updateGraph()

      let A = actions.analyse.map(a => this._analyse(a))
      let B = actions.evaluate.map(a => {
        let cell = graph.getCell(a.id)
        // This is necessary because we make sure the cell still exists
        if (cell) {
          if (this._canRunCell(cell)) {
            return this._evaluate(a)
          } else {
            // otherwise keep this as a next action
            a.suspended = true
            this._nextActions.set(a.id, a)
            return false
          }
        } else {
          return false
        }
      })
      res = A.concat(B)
    } else if (graph.needsUpdate()) {
      this._updateGraph()
    }
    return res
  }

  getNextActions() {
    return this._nextActions
  }

  _registerResource(doc) {
    const id = doc.id
    if (this._docs.hasOwnProperty(id)) throw new Error(`document with id ${id} already exists`)
    this._docs[id] = doc
    doc._registerCells()
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
    this._updateCell(cell.id, {})
  }

  /*
    Removes a cell from the engine.
  */
  _unregisterCell(cellOrId) { // eslint-disable-line
    let id = isString(cellOrId) ? cellOrId : cellOrId.id
    this._graph.removeCell(id)
  }

  _updateCell(id, cellData) {
    const graph = this._graph
    let cell = graph.getCell(id)
    Object.assign(cell, cellData)
    cell.status = UNKNOWN
    this._nextActions.set(id, {
      id,
      type: 'analyse',
      cellData
    })
  }

  _sendUpdate(updatedCells) {
    // TODO: this should send a batch update over to the app
    // and for testing this method should be 'spied'
    // updatedCells.forEach(cell => {
    //   console.log(`Updated cell ${cell.id}: ${cell._getStatusString()}`)
    // })
    this.emit('update', updatedCells)
  }

  _updateGraph() {
    const graph = this._graph
    let updatedIds = graph.update()
    let updatedCells = []
    updatedIds.forEach(id => {
      let cell = graph.getCell(id)
      if (cell) {
        // WIP: adding support for RangeCells
        // Instead of registering an evaluation, we just update the graph.
        // TODO: this requires another cycle to propagate the result of the RangeCell,
        // which would not be necessary in theory
        if (cell.status === READY) {
          if (cell instanceof RangeCell) {
            let value = this._getValueForRange(cell)
            graph.setValue(cell.id, value)
          } else {
            this._nextActions.set(cell.id, {
              type: 'evaluate',
              id: cell.id
            })
          }
        }
        if (!cell.hidden) {
          updatedCells.push(cell)
        }
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
    cell.errors = []
    // in case of constants, casting the string into a value,
    // updating the cell graph and returning without further evaluation
    if (cell.isConstant()) {
      // TODO: use the preferred type from the sheet
      let preferredType = 'any'
      let value = valueFromText(preferredType, cell.source)
      graph.setValue(id, value)
      return
    }
    // TODO: we need to reset the cell status. Should we let CellGraph do this?
    cell.status = UNKNOWN
    // otherwise the cell source is assumed to be dynamic source code
    const transpiledSource = cell.transpiledSource
    const lang = cell.getLang()
    return this._getContext(lang)
    .then(res => {
      // stop if this was aboreted or there is already a new action for this id
      if (this._nextActions.has(id)) {
        // console.log('action has been superseded')
        return
      }
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
      // stop if this was aboreted or there is already a new action for this id
      if (!res) return
      if (this._nextActions.has(id)) {
        // console.log('action has been superseded')
        return
      }
      // Note: treating all errors coming from analyseCode() as SyntaxErrors
      // TODO: we might want to be more specific here
      if (res.messages && res.messages.length > 0) {
        // TODO: we should not need to set this manually
        cell.status = ANALYSED
        graph.addErrors(id, res.messages.map(err => {
          return new SyntaxError(err.message)
        }))
      } else {
        // console.log('analysed cell', cell, res)
        // transform the extracted symbols into fully-qualified symbols
        // e.g. in `x` in `sheet1` is compiled into `sheet1.x`
        let { inputs, output } = this._compile(res, cell)
        this._nextActions.set(id, {
          type: 'register',
          id,
          inputs,
          output
        })
      }
    })
  }

  _evaluate(action) {
    const graph = this._graph
    const id = action.id
    const cell = graph.getCell(id)
    // TODO: is it really ok to wipe all the errors?
    cell.errors = []
    // console.log('evaluating cell', cell.toString())
    const lang = cell.getLang()
    let transpiledSource = cell.transpiledSource
    // EXPERIMENTAL: remove 'autorun'
    delete cell.autorun
    return this._getContext(lang)
    .then(res => {
      if (this._nextActions.has(id)) {
        // console.log('action has been superseded')
        return
      }
      if (res instanceof Error) {
        const msg = `Could not get context for ${lang}`
        console.error(msg)
        let err = new ContextError(msg, { lang })
        graph.addError(id, err)
      } else {
        const context = res
        // console.log('EXECUTING cell', cell.id, transpiledSource)
        // Catching errors here and turn them into a runtime error
        try {
          let inputs = this._getInputValues(cell.inputs)
          return context.executeCode(transpiledSource, inputs)
        } catch (err) {
          graph.addError(id, new RuntimeError(err.message, err))
        }
      }
    })
    .then(res => {
      // stop if this was aboreted or there is already a new action for this id
      if (!res) return
      if (this._nextActions.has(id)) {
        // console.log('action has been superseded')
        return
      }
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
      let qualifiedId = _qualifiedId(_scopeId, name)
      const symbol = new CellSymbol(type, qualifiedId, _scopeId, name, mangledStr)
      inputs.add(symbol)
    })
    // turn the output into a qualified id
    let output
    if (res.output) output = _qualifiedId(scopeId, res.output)
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

  _lookupDocumentId(name) {
    for (var id in this._docs) { // eslint-disable-line guard-for-in
      let doc = this._docs[id]
      if (doc.name === name || id === name) {
        return doc.id
      }
    }
  }

  _lookupDocument(name) {
    let docId = this._lookupDocumentId(name)
    return this._docs[docId]
  }

  /*
    Gathers the value for a cell range
    - `A1:A1`: value
    - `A1:A10`: array
    - `A1:E1`: array
    - `A1:B10`: table

    TODO: we should try to avoid using specific coercion here
  */
  _getValueForRange(rangeCell) {
    const { startRow, endRow, startCol, endCol } = rangeCell
    // TODO: rangeCell.docId is not accurate; it seems that this is the rather 'name'
    let sheet = this._lookupDocument(rangeCell.docId)
    let matrix = sheet.getCells()
    let val
    // range is a single cell
    // NOTE: with the current implementation of parseSymbol this should not happen
    /* istanbul ignore if */
    if (startRow === endRow && startCol === endCol) {
      val = getCellValue(matrix[startRow][startCol])
    }
    // range is 1D
    else if (startRow === endRow) {
      let cells = matrix[startRow].slice(startCol, endCol+1)
      val = this._getArrayValueForCells(cells)
    }
    else if (startCol === endCol) {
      let cells = []
      for (let i = startRow; i <= endRow; i++) {
        cells.push(matrix[i][startCol])
      }
      val = this._getArrayValueForCells(cells)
    }
    // range is 2D (-> creating a table)
    else {
      let data = {}
      for (let j = startCol; j <= endCol; j++) {
        let name = sheet.getColumnName(j) || getColumnLabel(j)
        let cells = []
        for (let i = startRow; i <= endRow; i++) {
          cells.push(matrix[i][j])
        }
        // TODO: why is it necessary to extract the primitive value here, instead of just using getCellValue()?
        data[name] = cells.map(c => {
          let val = getCellValue(c)
          if (val) {
            return val.data
          } else {
            return undefined
          }
        })
      }
      val = {
        // Note: first 'type' is for packing
        // and second type for diambiguation against other complex types
        type: 'table',
        data: {
          type: 'table',
          data,
          columns: endCol-startCol+1,
          rows: endRow-startRow+1
        }
      }
    }
    return val
  }

  _getArrayValueForCells(cells) {
    // TODO: we should try to decouple this implementation from
    // the rest of the application.
    // this is related to the Stencila's type system
    // Either, the engine is strongly coupled to the type system
    // or we need to introduce an abstraction.
    return gather('array', cells.map(c => getCellValue(c)))
  }

  _canRunCell(cell) {
    if (cell.hasOwnProperty('autorun')) {
      return cell.autorun
    }
    return cell.doc.autorun
  }

  _allowRunningCellAndPredecessors(id) {
    const graph = this._graph
    let predecessors = graph._getPredecessorSet(id)
    this._allowRunningCell(id)
    predecessors.forEach(_id => {
      this._allowRunningCell(_id)
    })
  }

  _allowRunningCell(id) {
    const graph = this._graph
    let cell = graph.getCell(id)
    cell.autorun = true
    let action = this._nextActions.get(id)
    if (action) {
      delete action.suspended
    }
  }

  _allowRunningAllCellsOfDocument(docId) {
    const graph = this._graph
    let doc = this._docs[docId]
    let cells = doc.getCells()
    if (doc instanceof Sheet) {
      cells = flatten(cells)
    }
    let ids = new Set()
    cells.forEach(cell => {
      ids.add(cell.id)
    })
    cells.forEach(cell => {
      graph._getPredecessorSet(cell.id, ids)
    })
    ids.forEach(id => {
      this._allowRunningCell(id)
    })
  }
}

/*
  Engine's internal model of a Document.
*/
class Document {

  constructor(engine, data) {
    this.engine = engine
    this.data = data
    if (!data.id) throw new Error("'id' is required")
    this.id = data.id
    this.name = data.name
    this.lang = data.lang || 'mini'
    if (data.hasOwnProperty('autorun')) {
      this.autorun = data.autorun
    } else {
      // TODO: using manual execution as a default for now
      this.autorun = true
    }
    this.cells = data.cells.map(cellData => this._createCell(cellData))
    // registration hook used for propagating initial cell state to the application
    if (data.onCellRegister) this.onCellRegister = data.onCellRegister
  }

  get type() { return 'document' }

  getCells() {
    return this.cells
  }

  setAutorun(val) {
    this.autorun = val
  }

  insertCellAt(pos, cellData) {
    let cell = this._createCell(cellData)
    this._registerCell(cell)
    this.cells.splice(pos, 0, cell)
    return cell
  }

  removeCell(id) {
    const qualifiedId = _qualifiedId(this.id, id)
    const cells = this.cells
    let pos = cells.findIndex(cell => cell.id === qualifiedId)
    if (pos >= 0) {
      let cell = cells[pos]
      this.cells.splice(pos,1)
      this.engine._unregisterCell(cell)
    } else {
      console.error('Unknown cell', id)
    }
  }

  updateCell(id, cellData) {
    let qualifiedId = _qualifiedId(this.id, id)
    if (isString(cellData)) {
      cellData = { source: cellData }
    }
    this.engine._updateCell(qualifiedId, cellData)
  }

  onCellRegister(cell) { // eslint-disable-line
  }

  _createCell(cellData) {
    if (isString(cellData)) {
      let source = cellData
      cellData = {
        id: uuid(),
        docId: this.id,
        source,
        lang: this.lang
      }
    }
    return new Cell(this, cellData)
  }

  _registerCell(cell) {
    const engine = this.engine
    engine._registerCell(cell)
    this.onCellRegister(cell)
  }

  _registerCells(block) {
    if (!block) block = this.cells
    block.forEach(cell => this._registerCell(cell))
  }
}

/*
  Engine's internal model of a Spreadsheet.
*/
class Sheet {

  constructor(engine, data) {
    this.engine = engine
    const docId = data.id
    if (!docId) throw new Error("'id' is required")
    this.id = docId
    this.name = data.name
    // default language
    const defaultLang = data.lang || 'mini'
    this.lang = defaultLang
    if (data.hasOwnProperty('autorun')) {
      this.autorun = data.autorun
    } else {
      // TODO: using auto/ cells automatically by default
      this.autorun = true
    }
    this.autorun = true
    // TODO: we can revise this as we move on
    // for now, data.cells must be present being a sequence of rows of cells.
    // data.columns is optional, but if present every data row have corresponding dimensions
    if (!data.cells) throw new Error("'cells' is mandatory")
    let ncols
    if (data.columns) {
      this.columns = data.columns
    } else {
      ncols = data.cells[0].length
      let columns = []
      for (let i = 0; i < ncols; i++) {
        columns.push({ type: 'auto' })
      }
      this.columns = columns
    }
    ncols = this.columns.length
    this.cells = data.cells.map((rowData) => {
      if (rowData.length !== ncols) throw new Error('Invalid data')
      return rowData.map(cellData => this._createCell(cellData))
    })
    this._initializeOutputs()

    if (data.onCellRegister) this.onCellRegister = data.onCellRegister
  }

  get type() { return 'sheet' }

  setAutorun(val) {
    this.autorun = val
  }

  getColumnName(colIdx) {
    let columnMeta = this.columns[colIdx]
    if (columnMeta && columnMeta.name) {
      return columnMeta.name
    } else {
      return getColumnLabel(colIdx)
    }
  }

  getCells() {
    return this.cells
  }

  queryCells(range) {
    return queryCells(this.cells, range)
  }

  updateCell(id, cellData) {
    let qualifiedId = _qualifiedId(this.id, id)
    if (isString(cellData)) {
      cellData = { source: cellData }
    }
    this.engine._updateCell(qualifiedId, cellData)
  }

  insertRows(pos, dataBlock) {
    // TODO: what if all columns and all rows had been removed
    let ncols = this.columns.length
    let block = dataBlock.map((rowData) => {
      if (rowData.length !== ncols) throw new Error('Invalid data')
      return rowData.map(cellData => this._createCell(cellData))
    })
    this.cells.splice(pos, 0, ...block)
    this._registerCells(block)
    this._updateCellOutputs(pos, 0)
  }

  deleteRows(pos, count) {
    let block = this.cells.slice(pos, pos+count)
    this.cells.splice(pos, count)
    this._unregisterCells(block)
    this._updateCellOutputs(pos, 0)
  }

  insertCols(pos, dataBlock) {
    const N = this.cells.length
    if (dataBlock.length !== N) throw new Error('Invalid dimensions')
    if (dataBlock.length === 0) return
    let m = dataBlock[0].length
    let block = dataBlock.map((rowData) => {
      if (rowData.length !== m) throw new Error('Invalid data')
      return rowData.map(cellData => this._createCell(cellData))
    })
    let cols = []
    for (let i = 0; i < m; i++) {
      cols.push({ type: 'auto' })
    }
    this.columns.splice(pos, 0, ...cols)
    for (let i = 0; i < N; i++) {
      let row = this.cells[i]
      row.splice(pos, 0, ...block[i])
    }
    this._registerCells(block)
    this._updateCellOutputs(0, pos)
  }

  deleteCols(pos, count) {
    const N = this.cells.length
    let block = []
    this.columns.splice(pos, count)
    for (var i = 0; i < N; i++) {
      let row = this.cells[i]
      block.push(row.slice(pos, pos+count))
      row.splice(pos, count)
    }
    this._unregisterCells(block)
    this._updateCellOutputs(0, pos)
  }

  onCellRegister(cell) { // eslint-disable-line
  }

  // This must be called after structural changes to update
  // the output symbol a cell which is derived from its position in the sheet
  // i.e. `cells[0][0]` in `sheet1` is associated to symbol `sheet1!A1`
  _updateCellOutputs(startRow, startCol) {
    const graph = this.engine._graph
    const cells = this.cells
    const N = cells.length
    const M = this.columns.length
    for (let i = startRow; i < N; i++) {
      for (let j = startCol; j < M; j++) {
        let cell = cells[i][j]
        if (cell.output) {
          graph._deregisterOutput(cell.id, cell.output)
        }
      }
    }
    for (let i = startRow; i < N; i++) {
      for (let j = startCol; j < M; j++) {
        let cell = cells[i][j]
        cell.output = this._getCellSymbol(i, j)
        graph._registerOutput(cell.id, null, cell.output)
      }
    }
  }

  _initializeOutputs() {
    const cells = this.cells
    const N = cells.length
    const M = this.columns.length
    for (let i = 0; i < N; i++) {
      for (let j = 0; j < M; j++) {
        let cell = cells[i][j]
        let output = this._getCellSymbol(i, j)
        cell.output = output
      }
    }
  }

  _getCellSymbol(rowIdx, colIdx) {
    return `${this.id}!${getCellLabel(rowIdx, colIdx)}`
  }

  _createCell(cellData) {
    // simple format: just the expression
    if (isString(cellData)) {
      let source = cellData
      cellData = {
        id: uuid(),
        docId: this.id,
        source,
      }
    }
    let cell = new Cell(this, cellData)
    return cell
  }

  _registerCell(cell) {
    const engine = this.engine
    engine._registerCell(cell)
    this.onCellRegister(cell)
  }

  _unregisterCell(cell) {
    const engine = this.engine
    engine._unregisterCell(cell)
  }

  _registerCells(block) {
    if (!block) block = this.cells
    block.forEach(row => row.forEach(cell => this._registerCell(cell)))
  }

  _unregisterCells(block) {
    if (!block) block = this.cells
    block.forEach(row => row.forEach(cell => this._unregisterCell(cell)))
  }
}

/*
  An internal cell that is used to represent range expressions such as `A1:B10`.
  This cell is used as proxy with a canonical expansion to primitive cells.
  I.e. `A1:B10` is proxied to `['A1',...,'B10']
*/
class RangeCell {

  constructor(symbol) {
    this.symbol = symbol
    this.id = symbol.id
    this.docId = symbol.scope
    this.inputs = new Set()
    this.output = symbol.id

    // managed by CellGraph
    this.status = ANALYSED
    this.value = undefined

    this.refs = 0

    this._initialize()
  }

  get hidden() { return true }

  hasError() { return false }

  hasErrors() { return false }

  clearErrors() {}

  addErrors() {}

  getValue() { return this.value }

  _initialize() {
    const docId = this.docId
    const name = this.symbol.name
    let [start, end] = name.split(':')
    let [startRow, startCol] = getRowCol(start)
    let [endRow, endCol] = getRowCol(end)
    if (startRow > endRow) ([startRow, endRow] = [endRow, startRow])
    if (startCol > endCol) ([startCol, endCol] = [endCol, startCol])

    this.startRow = startRow
    this.endRow = endRow
    this.startCol = startCol
    this.endCol = endCol

    for (let i = startRow; i <= endRow; i++) {
      for (let j = startCol; j <= endCol; j++) {
        let cellName = getCellLabel(i, j)
        let id = _qualifiedId(docId, cellName)
        // Note: it should be fine to use a string here, because we do not need
        // to do any deeper reflection, and just use it as a key
        this.inputs.add(id)
      }
    }
  }
}

function getCellValue(cell) {
  return cell ? cell.value : undefined
}
