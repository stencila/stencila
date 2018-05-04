import { isString, EventEmitter, flatten } from 'substance'
import { ContextError, RuntimeError, SyntaxError } from './CellErrors'
import { UNKNOWN, ANALYSED, READY, toInteger as statusToInt } from './CellStates'
import CellSymbol from './CellSymbol'
import { gather } from '../value'
import { valueFromText, getColumnLabel, qualifiedId as _qualifiedId } from '../shared/cellHelpers'
import EngineCellGraph from './EngineCellGraph'
import Sheet from './Sheet'
import Document from './Document'

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

  TODO: the former approach using special internal cells as proxy to
  cell ranges turned out to be cumbersome and error prone.
  Now I want to approach this differently, by adding a special means to lookup
  cell symbols and to propagate cell states for these.

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
    this._graph = new EngineCellGraph(this)

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

  hasResource(id) {
    return this._docs.hasOwnProperty(id)
  }

  getResource(id) {
    return this._docs[id]
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
        let cell = graph.getCell(a.id)
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
    let cell = this._graph.getCell(id)
    if (cell) {
      this._graph.removeCell(id)
    }
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

  _sendUpdate(type, cells) {
    let cellsByDocId = {}
    cells.forEach(cell => {
      let _cells = cellsByDocId[cell.docId]
      if (!_cells) _cells = cellsByDocId[cell.docId] = []
      _cells.push(cell)
    })
    this.emit('update', type, cellsByDocId)
  }

  _updateGraph() {
    const graph = this._graph
    let updatedIds = graph.update()
    let cells = new Set()
    updatedIds.forEach(id => {
      let cell = graph.getCell(id)
      if (cell) {
        // WIP: adding support for RangeCells
        // Instead of registering an evaluation, we just update the graph.
        // TODO: this requires another cycle to propagate the result of the RangeCell,
        // which would not be necessary in theory
        if (cell.status === READY) {
          this._nextActions.set(cell.id, {
            type: 'evaluate',
            id: cell.id
          })
        }
        cells.add(cell)
      }
    })
    if (cells.size > 0) {
      this._sendUpdate('state', cells)
    }
  }

  _analyse(action) {
    const graph = this._graph
    const id = action.id
    const cell = graph.getCell(id)
    // clear all errors which are not managed by the CellGraph
    cell.clearErrors(e => {
      return e.type !== 'graph'
    })
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
          // Note: these symbols are in plain-text analysed by the context
          // based on the transpiled source
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
    cell.clearErrors(e => {
      return e.type !== 'graph'
    })
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
    const symbolMapping = cell.symbolMapping
    const docId = cell.docId
    let inputs = new Set()
    // Note: the inputs here are given as mangledStr
    // typically we have detected these already during transpilation
    // Let's wait for it to happen where this is not the case
    res.inputs.forEach(str => {
      // Note: during transpilation we identify some more symbols
      // which are actually not real variables
      // e.g. for `sum(A1:B10)` would detect 'sum' as a potential variable
      // due to the lack of language reflection at this point.
      let s = symbolMapping[str]
      if (!s) throw new Error('FIXME: a symbol has been returned by analyseCode which has not been tracked before')
      // if there is a scope given explicily try to lookup the doc
      // otherwise it is a local reference, i.e. within the same document as the cell
      let targetDocId = s.scope ? this._lookupDocumentId(s.scope) : docId
      inputs.add(new CellSymbol(s, targetDocId, cell))
    })
    // turn the output into a qualified id
    let output
    if (res.output) output = _qualifiedId(docId, res.output)
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
    inputs.forEach(s => {
      let val
      switch(s.type) {
        case 'cell': {
          let sheet = this._docs[s.docId]
          if (sheet) {
            let cell = sheet.cells[s.startRow][s.startCol]
            val = cell.value
          }
          break
        }
        case 'range': {
          let sheet = this._docs[s.docId]
          if (sheet) {
            val = _getValueForRange(sheet, s.startRow, s.startCol, s.endRow, s.endCol)
          }
          break
        }
        default:
          val = graph.getValue(s)
      }
      // Note: the transpiled source code is used for evaluation
      // thus we expose values via transpiled/mangled names here
      result[s.mangledStr] = val
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

  _canRunCell(cell) {
    if (cell.hasOwnProperty('autorun')) {
      return cell.autorun
    }
    return cell.doc.autorun
  }

  _allowRunningCellAndPredecessors(id) {
    const graph = this._graph
    let predecessors = graph._getPredecessorSet(id)
    this._allowRunningCell(id, true)
    predecessors.forEach(_id => {
      this._allowRunningCell(_id)
    })
  }

  _allowRunningCell(id, reset) {
    const graph = this._graph
    let cell = graph.getCell(id)
    cell.autorun = true
    if (reset && statusToInt(cell.status) > statusToInt(ANALYSED)) {
      cell.status = ANALYSED
      graph._structureChanged.add(id)
    }
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

function getCellValue(cell) {
  return cell ? cell.value : undefined
}

function _getArrayValueForCells(cells) {
  // TODO: we should try to decouple this implementation from
  // the rest of the application.
  // this is related to the Stencila's type system
  // Either, the engine is strongly coupled to the type system
  // or we need to introduce an abstraction.
  return gather('array', cells.map(c => getCellValue(c)))
}


/*
  Gathers the value for a cell range
  - `A1:A1`: value
  - `A1:A10`: array
  - `A1:E1`: array
  - `A1:B10`: table

  TODO: we should try to avoid using specific coercion here
*/
function _getValueForRange(sheet, startRow, startCol, endRow, endCol) {
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
    val = _getArrayValueForCells(cells)
  }
  else if (startCol === endCol) {
    let cells = []
    for (let i = startRow; i <= endRow; i++) {
      cells.push(matrix[i][startCol])
    }
    val = _getArrayValueForCells(cells)
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
