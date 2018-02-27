import { uuid, isEqual } from 'substance'
import CellGraph from './CellGraph'
import { ContextError, RuntimeError } from './CellErrors'
import { READY } from './CellStates'
import Cell from './Cell'

/*
  WIP
  The Engine will be run in a worker, together with a MiniContext and a JsContext


  Possible Errors:
  - language context is not available
  - cell expression has syntax errors
  - cell evaulation yields runtime errors (including validation)
*/
export default class Engine {

  constructor(host) {
    this._host = host
    this._graph = new CellGraph()

    // a list of change requests
    this._requests = []
    // for every (changed) cell there is information what to do next
    // not including cells that have not been changed
    // the order does not matter here, as dependencies are managed in the graph
    // Note: using a hash, because there can be only one valid next action for a single cell
    this._state = {}
    // the engine records any kind of changes first, and then decides
    // what to do actually, actions on the same cell can supersede each other
  }

  addCell(cellData) {
    let cell = new Cell(cellData)
    this._graph.addCell(cell)
    this._addRequest(cell.id, 'updateCell', { cell })
  }

  step() {
    // take all requests, consider superseded ones
    // then update the graph
    // then trigger actions
    // and update the app
  }

  _addRequest(id, action, data) {
  }

  _updateCell(cell) {
    const graph = this._graph
    // analyse the source code
    const id = cell.id
    const source = cell.source
    const docId = cell.docId
    // document | sheet
    // TODO: document cells can be multi-line, and have a special semantics for exposing variables
    // sheet expressions must be single-line and have a prefix `(ID)?=`
    // variables, simple expressions
    const lang = cell.lang
    let context, token

    return this._getContext(lang)
    .then(res => {
      if (res instanceof Error) {
        const msg = `Could not get context for ${lang}`
        console.error(msg)
        graph.addError(id, new ContextError(msg, { lang }))
        return
      }
      context = res
    })
    .then(() => {
      token = uuid()
      this._tokens[id] = token
      return context.analyseCode(source)
    })
    .then(res => {
      // TODO: we want to use 'hidden' cells with explicit dependencies
      // for range symbols (or column/row symbols)

      // console.log('ANALYSED cell', cell, res)
      // skip if this cell has been rescheduled in the meantime
      if (this._tokens[id] !== token) return
      // transform the extracted symbols into fully-qualified symbols
      // e.g. in `x` in `sheet1` is compiled into `sheet1.x`
      let { inputs, output } = this._compile(res, docId)
      const oldState = graph.getCell(id)
      let oldOutput = oldState.output
      let oldInputs = oldState.inputs
      // only update the graph if inputs or output have changed
      if (!isEqual(oldOutput, output)) {
        graph.setOutput(id, output)
      }
      if (!isEqual(oldInputs, inputs)) {
        graph.setInputs(id, inputs)
      }
    })
  }

  _compile(res, docId = 'global') {
    // TODO: create CellSymbol instances by 'parsing' the received string symbols
    let inputs = new Set()
    let output = undefined
    return { inputs, output }
  }

  _evaluate(cell) {
    console.log('evaluating cell', cell.id)
    const graph = this._graph
    const lang = cell.lang
    const source = cell.source
    let token
    this._getContext(lang)
    .then(context => {
      token = uuid()
      this._tokens[cell.id] = token
      // console.log('EXECUTING cell', cell.id, source)
      let inputs = this._getInputValues(cell.inputs)
      return context.executeCode(source, inputs)
    })
    .then(res => {
      if (!this._tokens[cell.id] === token) return
      // console.log('executed cell', cell.id, res)
      graph.setResult(cell.id, res.value, this._createRuntimeErrors(res.messages))
    })
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
      let val = graph.getValue(symbol)
      result[symbol] = val
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
}