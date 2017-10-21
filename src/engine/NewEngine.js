import { uuid } from 'substance'
import { getCellState } from '../shared/cellHelpers'
import { INITIAL, ANALYSED } from './CellState'
// HACK: using DocumentChange to communicate node state changes
import { DocumentChange } from 'substance'

export default class Engine {

  constructor(host) {
    this._host = host

    this._cells = {}

    // dependency graph
    this._ins = {}
    this._outs = {}

    // cell id -> token
    this._tokens = {}

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
        // console.log('ANALYSED cell', cell, res)

        cellState.state = ANALYSED
        cellState.inputs = res.inputs
        cellState.output = res.output
        cellState.messages = res.messages
        cellState.tokens = res.tokens
        cellState.nodes = res.nodes
        this._notifyCell(cell)
      })
    })
  }

  _notifyCell(cell) {
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
