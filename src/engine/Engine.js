import { uuid, isEqual } from 'substance'
import { getCellState, getCellValue, getCellLabel } from '../shared/cellHelpers'
import { INITIAL, ANALYSED, EVALUATED,
  PENDING, INPUT_ERROR, INPUT_READY,
  RUNNING, ERROR, OK,
  deriveCellStatus
} from './CellState'
// HACK: using DocumentChange to communicate node state changes
import { DocumentChange } from 'substance'
import CellGraph from './CellGraph'

export default class Engine {

  constructor(host) {
    this._host = host

    // dependency graph
    this._graph = new CellGraph()

    this._scopes = {}

    // used to make sure that an asynchronous
    // job gets ignored, if another job has been
    // scheduled instead
    this._tokens = {}

    // whenever a cell goes is ANALYSED and all inputs are ready
    // it will be added to candidates
    this._candidates = new Set()
  }

  registerDocument(uuid, doc) {
    this._graph.registerDocument(uuid, doc)
  }

  registerScope(name, uuid) {
    this._scopes[name] = uuid
  }

  registerCell(cell) {
    if (this._graph.contains(cell.id)) {
      throw new Error('Cell with the same id already exists.')
    }
    this._graph.addCell(cell)

    let cellState = getCellState(cell)
    cellState._engineState = INITIAL
    deriveCellStatus(cellState)
    this._notifyCells(cell.id)

    this._analyse(cell)
  }

  updateCell(cellId) {
    // console.log('updating cell', cellId)
    let cell = this._getCell(cellId)
    let cellState = getCellState(cell)
    cellState._engineState = INITIAL
    deriveCellStatus(cellState)
    this._notifyCells(cell.id)

    this._analyse(cell)
  }

  removeCell(cellId) {
    // TODO: need to reorganize the dep graph
    let cell = this._getCell(cellId)
    if (cell) {
      this._candidates.delete(cell)
      this._updateDependencies(cell)
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
        console.error('Could not get context for %s', lang)
        cellState.messages = [context]
        deriveCellStatus(cellState)
        this._notifyCells([cell.id])
        return
      }

      let token = uuid()
      this._tokens[cell.id] = token
      let source = cell.source || ''
      return context.analyseCode(source).then((res) => {
        console.log('ANALYSED cell', cell, res)
        // skip if this cell has been rescheduled in the meantime
        if (this._tokens[cell.id] !== token) return

        // takes local symbol names and compiles into
        // symbols for the shared value scope
        // e.g. variable 'x' in 'doc1' is compiled to 'doc1.x'
        // or 'A1:A2' is compiled to ['sheet1.A1', 'sheet1.A2']
        let { inputs, output } = this._compile(res.inputs, res.output, cell.docId)
        let oldOutput = cellState.output
        let oldInputs = cellState.inputs

        cellState._engineState = ANALYSED
        cellState.inputs = inputs
        cellState.output = output
        cellState.messages = res.messages
        cellState.tokens = res.tokens
        cellState.nodes = res.nodes
        deriveCellStatus(cellState)

        // FIXME: to be able to broadcast changes to cells
        // we need to make the DepGraph based on cell ids, not on symbols
        if (!isEqual(oldOutput, output) || !isEqual(oldInputs, inputs)) {
          this._graph.updateCell(cell)
        }

        // if there was no error before be
        if (cellState.status === PENDING) {
          // TODO: we should not implicitly schedule : this call also
          cellState.status = this._getInputState(cell)
          if (cellState.status === INPUT_READY) {
            this._candidates.add(cell)
          } else {
            this._candidates.delete(cell)
          }
        }
        this._updateDependencies(cell)

        this._triggerScheduler()
      })
    })
  }

  _compile(inputs, output, docId) {
    console.log('_compile', inputs, output, docId)
    let result = {
      inputs: inputs ? inputs.map((input) => {
        let _docId
        if (input.scope) {
          _docId = this._scopes[input.scope]
          if (!docId) {
            throw new Error(`Unknown document ${input.scope}`)
          }
        } else {
          _docId = docId
        }
        input.docId = _docId
        return input
      }) : []
    }
    if (output) {
      result.output = output
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
    let candidates = this._candidates
    if (this._running) {
      return
    }
    this._running = true
    try {
      while(candidates.size > 0) {
        this._step()
      }
    } finally {
      this._running = false
    }
  }

  _step() {
    const candidates = this._candidates
    if (candidates.size === 0) return
    let cell = candidates.values().next().value
    candidates.delete(cell)

    let cellId = cell.id
    // go through all candidates and evaluate when ready
    let cellState = getCellState(cell)
    if (cellState._engineState === EVALUATED) {
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
      cellState.status = RUNNING
      this._notifyCells([cell.id])

      context.executeCode(source, inputs).then((res) => {
        if (!this._tokens[cell.id] === token) return
        // console.log('executed cell', cell.id, res)
        // TODO: need better MiniContext to continue
        delete this._candidates[cellId]
        cellState._engineState = EVALUATED
        cellState.value = res.value
        // Note: we want to retain messages from
        // the code analysis, so we concat here
        cellState.messages = (cellState.messages || []).concat(res.messages)
        deriveCellStatus(cellState)

        this._updateDependencies(cell)

        this._triggerScheduler()
      })
    })
  }

  _getInputValues(cellId) {
    let graph = this._graph
    let cell = this._getCell(cellId)
    let cellState = getCellState(cell)
    let result = {}
    // TODO: for cross-references we must
    // mangle the name of a symbol
    cellState.inputs.forEach((symbol) => {
      switch (symbol.type) {
        case 'var': {
          let cell = graph.lookup(symbol)
          let val = getCellValue(cell)
          result[symbol.name] = val
          break
        }
        case 'cell': {
          let cell = graph.lookup(symbol)
          let val = getCellValue(cell)
          let name = getCellLabel(symbol.row, symbol.col)
          result[name] = val
          break
        }
        case 'range': {
          throw new Error('Not implemented yet.')
        }
        default:
          throw new Error('Invalid state')
      }
    })
    return result
  }

  _updateDependencies(cell) {
    const graph = this._graph
    let visited = {}
    let queue = [].concat(graph.getOutputs(cell.id))
    let dirty = [cell.id]
    while (queue.length > 0) {
      let next = queue.shift()
      if (visited[next.id]) continue
      if (next.isCell()) {
        let cellState = getCellState(next)
        let _state = cellState._engineState
        if (_state === ANALYSED || _state === EVALUATED) {
          cellState._engineState = ANALYSED
          cellState.status = this._getInputState(next)
          if (cellState.status === INPUT_READY) {
            this._candidates.add(next)
          } else {
            this._candidates.delete(next)
          }
        }
        dirty.push(next.id)
        queue = queue.concat(
          graph.getOutputs(next.id).filter(c => !visited[c.id])
        )
      }
      visited[cell.id] = true
    }
    this._notifyCells(...dirty)
  }

  _getInputState(cell) {
    let inputs = this._graph.getInputs(cell.id)
    let ready = true
    for (let i = 0; i < inputs.length; i++) {
      let input = inputs[i]
      if (!input) {
        console.error('FIXME: depending on an unregistered cell')
        return INPUT_ERROR
      }
      // TODO: we will have other type of dependencies, such as
      // cell-references or externally managed values
      if (input.isCell()) {
        let cellState = getCellState(input)
        if (cellState.status === ERROR || cellState.status === INPUT_ERROR) {
          return INPUT_ERROR
        } else if (cellState.status !== OK) {
          ready = false
        }
      } else {
        console.error('TODO: need to check the availability of inputs other than expression cells')
      }
    }

    return ready ? INPUT_READY : PENDING
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