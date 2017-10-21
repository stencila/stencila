import { uuid, isEqual } from 'substance'
import { getCellState } from '../shared/cellHelpers'
import { INITIAL, ANALYSED, EVALUATED } from './CellState'
// HACK: using DocumentChange to communicate node state changes
import { DocumentChange } from 'substance'
import DependencyGraph from './DependencyGraph'

export default class Engine {

  constructor(host) {
    this._host = host

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
    if (this._graph.contains(cell.id)) {
      throw new Error('Cell with the same id already exists.')
    }
    this._graph.addCell(cell)

    let cellState = getCellState(cell)
    cellState.state = INITIAL
    this._notifyCells(cell.id)

    this._analyse(cell)
  }

  updateCell(cellId) {
    // console.log('updating cell', cellId)
    let cell = this._getCell(cellId)
    let cellState = getCellState(cell)
    cellState.state = INITIAL
    this._notifyCells(cell.id)

    this._analyse(cell)
  }

  removeCell(cellId) {
    // TODO: need to reorganize the dep graph
    let cell = this._getCell(cellId)
    if (cell) {
      this._invalidateDependencies(cell)
      this._graph.removeCell(cellId)
    }
  }

  _getContext(lang) {
    return this._host.createContext(lang)
  }

  _getCell(cellId) {
    let cell = this._graph.getCell(cellId)
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
        this._notifyCells(cell.id)

        return Promise.resolve()
      }

      let token = uuid()
      this._tokens[cell.id] = token
      let source = cell.source || ''
      return context.analyseCode(source).then((res) => {
        // skip if this cell has been rescheduled in the meantime
        if (this._tokens[cell.id] !== token) return

        // console.log('ANALYSED cell', cell, res)
        // takes local symbol names and compiles into
        // symbols for the shared value scope
        // e.g. variable 'x' in 'doc1' is compiled to 'doc1.x'
        // or 'A1:A2' is compiled to ['sheet1.A1', 'sheet1.A2']
        let { inputs, output } = this._compile(res.inputs, res.output, cell.scope)
        let oldOutput = cellState.output
        let oldInputs = cellState.inputs

        cellState.state = ANALYSED
        cellState.inputs = inputs
        cellState.output = output
        cellState.messages = res.messages
        cellState.tokens = res.tokens
        cellState.nodes = res.nodes
        this._triggerScheduler()

        // FIXME: to be able to broadcast changes to cells
        // we need to make the DepGraph based on cell ids, not on symbols
        if (!isEqual(oldOutput, output) || !isEqual(oldInputs, inputs)) {
          this._graph.updateCell(cell)
        }

        this._candidates[cell.id] = true
        this._invalidateDependencies(cell)
      })
    })
  }

  _compile(inputs, output, scope) {
    let result = {
      inputs: []
    }
    if (inputs) {
      result.inputs = inputs.map((input) => {
        // TODO: complex symbols
        return scope ? `${scope}.${input}` : input
      })
    }
    if (output) {
      result.output = scope ? `${scope}.${output}` : output
    }
    return result
  }

  _triggerScheduler() {
    setTimeout(() => {
      // TODO: we should avoid that the scheduler gets stuck
      // because of exceptions somewhere in the code
      // maybe we should trigger scheduleEvaluation() via a background process
      this._scheduleEvaluation()
    })
  }

  _scheduleEvaluation() {
    let candidates = Object.keys(this._candidates)
    candidates.forEach((cellId) => {
      let cell = this._getCell(cellId)
      // go through all candidates and evaluate when ready
      if (cell && this._isReady(cell)) {
        let cellState = getCellState(cell)
        if (cellState.state === EVALUATED) {
          throw new Error('FIXME: retriggering an already evaluated cell')
        }
        let lang = cell.language
        let source = cell.source
        // remove this from candidates otherwise it will be re-evaluated infinitely
        delete this._candidates[cell.id]
        this._getContext(lang)
        .then((context) => {
          let token = uuid()
          this._tokens[cell.id] = token
          // console.log('EXECUTING cell', cell.id, source)

          // TODO: we want to force simple expression for Spreadsheet cells
          // We need to somehow 'transpile' cell and range expressions
          // and provide them using a generated symbol name
          let inputs = this._getInputValues(cellId)
          context.executeCode(source, inputs).then((res) => {
            if (!this._tokens[cell.id] === token) return
            // console.log('executed cell', cell.id, res)
            // TODO: need better MiniContext to continue
            delete this._candidates[cellId]
            cellState.state = EVALUATED
            cellState.value = res.value
            cellState.messages = res.messages

            this._triggerScheduler()

            // TODO: this should be named differently
            // it should do different things, such as
            // 'invalidating' evaluated cells
            // but also
            this._invalidateDependencies(cell)
          })
        })
      }
    })
  }

  _isReady(cell) {
    // TODO: go through all deps of the cell
    // and see if they have been evaluated and
    // not errored
    let inputs = this._graph.getInputs(cell.id)
    for (let i = 0; i < inputs.length; i++) {
      let input = inputs[i]
      if (!input) {
        console.error('FIXME: depending on an unregistered cell')
        return false
      }
      // TODO: we will have other type of dependencies, such
      // cell-references (A1) or externally managed values
      if (input.isCell()) {
        let cellState = getCellState(input)
        if (cellState.state !== EVALUATED || cellState.hasErrors()) {
          return false
        }
      } else {
        console.error('TODO: need to check the availability of inputs other than expression cells')
      }
    }
    return true
  }

  _getInputValues(cellId) {
    let result = {}
    this._graph.getInputs(cellId).forEach((input) => {
      if (input.isCell()) {
        let cellState = getCellState(input)
        let output = cellState.output
        if (output) {
          result[output] = cellState.value
        }
      } else {
        console.error('TODO: other input types?')
      }
    })
    return result
  }

  _invalidateDependencies(cell) {
    const graph = this._graph
    // invalidate all cells depending on this one
    // invalidate a cell / output
    // console.log('invalidate cell', cell)
    let visited = {}
    let queue = [].concat(graph.getOutputs(cell.id))
    let dirty = [cell.id]
    while (queue.length > 0) {
      let next = queue.shift()
      if (visited[next.id]) continue
      if (next.isCell()) {
        let cellState = getCellState(next)
        if (cellState.state === EVALUATED) {
          cellState.state = ANALYSED
          this._candidates[next.id] = true
          dirty.push(next.id)
          queue = queue.concat(
            graph.getOutputs(next.id).filter(c => !visited[c.id])
          )
        }
      }
      visited[cell.id] = true
    }
    this._notifyCells(...dirty)
  }

  _notifyCells(...cellIds) {
    if (cellIds.length === 0) return
    // Note: need to defer to avoid triggering
    // a reflow while already flowing
    setTimeout(() => {
      const editorSession = this.editorSession
      if (!editorSession) return
      editorSession._setDirty('document')
      editorSession._setDirty('commandStates')
      let change = new DocumentChange([], {}, {})
      change._extractInformation()
      cellIds.forEach((cellId) => {
        // console.log('notifying', cellId)
        change.updated[cellId] = true
      })
      editorSession._change = change
      editorSession._info = {}
      editorSession.startFlow()
    })
  }

}