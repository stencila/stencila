import { uuid, isEqual } from 'substance'
import { getCellState } from '../shared/cellHelpers'
import { INITIAL, ANALYSED, EVALUATED } from './CellState'
// HACK: using DocumentChange to communicate node state changes
import { DocumentChange } from 'substance'
import DependencyGraph from './DependencyGraph'

export default class Engine {

  constructor(host) {
    this._host = host

    this._cells = {}

    // table to lookup cells by
    // symbol (i.e. var or cell)
    // TODO: we need to introduce scopes
    // to allow for inter sheet references
    this._lookupTable = {}

    // dependency graph
    this._graph = new DependencyGraph()

    // used to make sure that an asynchronous
    // job gets ignored, if another job has been
    // scheduled instead
    this._tokens = {}

    // whenever a cell goes into ANALYSED
    // it will be added here
    this._candidates = {}
  }

  addCell(cell) {
    if (this._cells[cell.id]) {
      throw new Error('Cell with the same id already exists.')
    }
    this._cells[cell.id] = cell

    let cellState = getCellState(cell)
    cellState.state = INITIAL
    this._notifyCell(cell)

    this._analyse(cell)
  }

  updateCell(cellId) {
    // console.log('updating cell', cellId)
    let cell = this._getCell(cellId)
    let cellState = getCellState(cell)
    cellState.state = INITIAL
    this._notifyCell(cell)

    this._analyse(cell)
  }

  removeCell(cellId) {
    // TODO: need to reorganize the dep graph
    delete this._cells[cellId]
  }

  _getContext(lang) {
    return this._host.createContext(lang)
  }

  _getCell(cellId) {
    let cell = this._cells[cellId]
    if (!cell) throw new Error(`Unknown cell ${cellId}`)
    return cell
  }

  _analyse(cell) {
    let cellState = getCellState(cell)
    let lang = cell.language
    this._getContext(lang)
    .then((context) => {
      if (context instanceof Error) {
        cellState.messages = [context]
        this._notifyCell(cell)
        return Promise.resolve()
      }

      let token = uuid()
      this._tokens[cell.id] = token
      let source = cell.source || ''
      return context.analyseCode(source).then((res) => {
        if (this._tokens[cell.id] !== token) return
        console.log('ANALYSED cell', cell, res)
        // takes local symbol names and compiles into
        // symbols for the shared value scope
        // e.g. variable 'x' in 'doc1' is compiled to 'doc1.x'
        // or 'A1:A2' is compiled to ['sheet1.A1', 'sheet1.A2']
        let { inputs, output } = this._compile(res.inputs, res.output, cell.scope)
        let oldOutput = cellState.output
        let oldInputs = cellState.inputs
        if (oldOutput && oldOutput !== output) {
          this._graph._removeResource(oldOutput)
          this._graph._update()
          this._lookupTable.delete(oldOutput)
        } else if (!isEqual(oldInputs, inputs)) {
          this._graph._setDependencies(output, inputs)
          this._graph._update()
        }
        this._candidates[cell.id] = true
        cellState.state = ANALYSED
        cellState.inputs = inputs
        cellState.output = output
        cellState.messages = res.messages
        cellState.tokens = res.tokens
        cellState.nodes = res.nodes
        this._notifyCell(cell)

        if (output) {
          this._lookupTable[output] = cell.id
        }
        this._scheduleEvaluation()
      })
    })
  }

  _compile(inputs, output, scope) {
    let result = {
      inputs: []
    }
    if (inputs) {
      result.inputs = inputs.map((input) => {
        // TODO: we need to support more complex symbols
        return scope ? `${scope}.${input}` : input
      })
    }
    if (output) {
      result.output = scope ? `${scope}.${output}` : output
    }
    return result
  }

  _invalidate() {
    // invalidate a cell / output
  }

  _scheduleEvaluation() {
    let candidates = Object.keys(this._candidates)
    for (let i = 0; i < candidates.length; i++) {
      let cellId = candidates[i]
      let cell = this._getCell(cellId)
      // go through all candidates and evaluate
      // when ready
      if (cell && this._isReady(cell)) {
        let cellState = getCellState(cell)
        let lang = cell.language
        let source = cell.source
        // remove this from candidates to avoid
        // being retriggered withtout being changed
        delete this._candidates[cell.id]
        this._getContext(lang)
        .then((context) => {
          let token = uuid()
          this._tokens[cell.id] = token
          console.log('EXECUTING cell', cell.id, source)

          // TODO: we want to force simple expression for Spreadsheet cells
          // We need to somehow 'transpile' cell and range expressions
          // and provide them using a generated symbol name
          let inputs = this._resolveInputs(cellState.inputs)
          context.executeCode(source, inputs).then((res) => {
            if (!this._tokens[cell.id] === token) return
            console.log('executed cell', cell.id, res)
            // TODO: need better MiniContext to continue
            cellState.state = EVALUATED
            cellState.value = res.value
            cellState.messages = res.messages
            // cellState.messages = res.messages
            this._notifyCell(cell)

            this._scheduleEvaluation()
          })
        })
      }
    }
  }

  _isReady(cell) {
    // TODO: go through all deps of the cell
    // and see if they have been evaluated and
    // not errored
    let cellState = getCellState(cell)
    let inputs = cellState.inputs
    for (let i = 0; i < inputs.length; i++) {
      let input = this._lookup(inputs[i])
      if (!input) {
        console.error('FIXME: depending on an unregistered cell')
        return false
      }
      // TODO: we will have other type of dependencies, such
      // cell-references (A1) or externally managed values
      if (input.isCell()) {
        let cellState = getCellState(input)
        if (cellState.state !== EVALUATED) {
          return false
        }
        if (cellState.hasErrors()) {
          return false
        }
      } else {
        console.error('TODO: need to check the availability of inputs other than expression cells')
      }
    }
    return true
  }

  _lookup(symbol) {
    // TODO: at some point we will introduce more complex symbols
    // such as cell-references or inputs
    let id = this._lookupTable[symbol]
    return this._getCell(id)
  }

  _resolveInputs(inputs) {
    let result = {}
    inputs.forEach((symbol) => {
      // TODO: we need complex symbols
      let name = symbol
      let res = this._lookup(name)
      let val
      if (res.isCell()) {
        val = getCellState(res).value
      }
      result[name] = val
    })
    return result
  }

  _notifyCell(cell) {
    // Note: need to defer to avoid triggering
    // a reflow while already flowing
    setTimeout(() => {
      const editorSession = this.editorSession
      if (!editorSession) return
      editorSession._setDirty('document')
      editorSession._setDirty('commandStates')
      let change = new DocumentChange([], {}, {})
      change._extractInformation()
      change.updated[cell.id] = true
      editorSession._change = change
      editorSession._info = {}
      editorSession.startFlow()
    })
  }

}

/*

  // we are storing the graph as a mapping from
  // cellId -> { inputs, output }
  // and reconstruct the whole graph after changes
  // (can be optimized later)
  _buildDependencyGraph() {
    let cells = this._cells
    // cellId -> [successors]
    let outs = {}
    // cellId -> [predecessors]
    let ins = {}
    // mapping from qualified name to cellId
    let byName = {}

    // DFS to detect cyclic deps
    let visited = {}
    forEach(this._cells, cell => this.__buildDependencyGraph(cell, visited))

    this._ins = ins
    this._outs = outs
  }

  __buildDependencyGraph(cell, visited) {
    // create a slot in outs
    if (visited[cell.id] === -1) {
      throw new Error('Found cyclic dependency')
    }
    if (visited[cell.id]) return
    visited[cell.id] = -1
    // TODO: requires successful code analysis
    let name = cell.getName()
    if (name) {
      byName[name] = cell.id
    }
    outs[cell.id] = new Set()
    // TODO: cell.inputs should not just be names
    // but rather records (var, cell, range)
    // in case of 'range', input expands to a list of ids
    cell.inputs.forEach((input) => {
      let prevIds = this._mapInput(input)
      prevIds.forEach(prevIds, (prevId) => {
        let prev = cells[prevId]
        if (!prev) {
          // ohoh... could not find cell
          return
        }
        if (!ins[prevId]) {
          ins[prevId] = new Set()
        }
        ins[prevId].add(cell.id)
        this.__buildDependencyGraph(prev)
      })
    })
    visited[cell.id] = true
  }

  _mapInput(input) {
    let ids = []
    switch(input.type) {
      case 'var': {

        break
      }
      case 'cell': {

        break
      }
      case 'range': {

        break
      }
    }
    return ids
  }

  _getTrace(cellId) {
    // TODO: we could cache such things
    // and invalidate when cells are changed
    let trace = []
    this._traverseDependencies(cellId, (cell) => {
      trace.push(cell)
    })
    return trace
  }

  _traverseDependencies(id, fn, visited = {}) {
    // compute the schedule first
    const outs = this._outs
    let queue = [id]
    while(queue.length > 0) {
      let nextId = queue.shift()
      if (visited[nextId] === -1) {
        throw new Error('Cyclic dependency')
      }
      if (visited[nextId]) continue
      visited[nextId] = -1
      let next = this._getEntry(nextId)
      if (!next) {
        // next is not an entry
      } else {
        fn(next)
      }
      visited[nextId] = true
      let out = outs[id]
      if (out && out.size>0) {
        queue = queue.concat(...out)
      }
    }
  }

*/
