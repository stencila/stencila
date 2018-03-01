import { uuid } from 'substance'
import CellGraph from './CellGraph'
import { ContextError, RuntimeError } from './CellErrors'
import { READY } from './CellStates'
import Cell from './Cell'
import CellSymbol from './CellSymbol'
import { parseSymbol } from '../shared/expressionHelpers'

/*
  WIP
  The Engine will be run in a worker, together with a MiniContext and a JsContext
*/
export default class Engine {

  constructor(host) {
    this._host = host
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

  addCell(cellData) {
    this.updateCell(cellData.id, cellData)
  }

  updateCell(id, cellData) {
    this._nextActions.set(cellData.id, {
      id,
      type: 'analyse',
      cellData,
      // used to detect invalidations
      token: uuid(),
    })
  }

  cycle() {
    const nextActions = this._nextActions
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
    const graph = this._graph
    actions.update.forEach(a => {
      if (a.errors && a.errors.length > 0) {
        graph.addErrors(a.id, a.errors)
      } else {
        graph.setValue(a.id, a.value)
      }
    })
    actions.register.forEach(a => {
      if (!graph.hasCell(a.id)) {
        let cell = new Cell(a.cellData)
        graph.addCell(cell)
      }
      if (a.inputs) graph.setInputs(a.id, a.inputs)
      if (a.output) graph.setOutput(a.id, a.output)
    })

    let updatedIds = graph.update()
    let updatedCells = []
    updatedIds.forEach(id => {
      let cell = graph.getCell(id)
      if (cell) {
        if (cell.state === READY) {
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

    let A = actions.analyse.map(a => this._analyse(a))
    let B = actions.evaluate.map(a => this._evaluate(a))
    return A.concat(B)
  }

  hasNextAction() {
    return this._nextActions.size > 0
  }

  getNextActions() {
    return this._nextActions
  }

  _sendUpdate() {
    // TODO: this should send a batch update over to the app
    // and for testing this method should be 'spied'
  }

  _analyse(action) {
    const graph = this._graph
    const id = action.id
    const cellData = action.cellData
    const { source, docId, lang } = cellData
    return this._getContext(lang)
    .then(res => {
      if (res instanceof Error) {
        const msg = `Could not get context for ${lang}`
        console.error(msg)
        let err = new ContextError(msg, { lang })
        graph.addError(id, err)
      } else {
        const context = res
        return context.analyseCode(source)
      }
    })
    .then(res => {
      if (!res) return
      console.log('ANALYSED cell', id, res)
      // transform the extracted symbols into fully-qualified symbols
      // e.g. in `x` in `sheet1` is compiled into `sheet1.x`
      let { inputs, output } = this._compile(res, docId)
      this._nextActions.set(id, {
        type: 'register',
        id,
        inputs,
        output,
        cellData
      })
    })
  }

  _evaluate(action) {
    const graph = this._graph
    const id = action.id
    const cell = graph.getCell(id)
    console.log('evaluating cell', id)
    const { lang, source } = cell
    return this._getContext(lang)
    .then(res => {
      if (res instanceof Error) {
        const msg = `Could not get context for ${lang}`
        console.error(msg)
        let err = new ContextError(msg, { lang })
        graph.addError(id, err)
      } else {
        const context = res
        console.log('EXECUTING cell', cell.id, source)
        let inputs = this._getInputValues(cell.inputs)
        return context.executeCode(source, inputs)
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

  _compile(res, scopeId = 'global') {
    // TODO: create CellSymbol instances by 'parsing' the received string symbols
    let inputs = new Set()
    res.inputs.forEach(str => {
      const { type, scope, name, mangledStr } = parseSymbol(str)
      // if there is a scope given explicily try to lookup the doc
      if (scope) {
        // trying to lookup the given scope
        // if that fails then we just pass through the
        // given scope identifier
        // which very likely will result in an 'UnresolvedInputError'
        scopeId = this._lookupDocument(scope) || scopeId
      }
      let qualifiedId = scopeId + '!' + name
      const symbol = new CellSymbol(type, qualifiedId, scopeId, name, mangledStr)
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
      let val = graph.getValue(symbol)
      // use the mangled symbol string, to expose
      // the value for evaluation
      // This representation is particularly valid only
      // within the transpiled cell code
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
}