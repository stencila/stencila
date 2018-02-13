import { flatten, isString, isArray } from 'substance'
import { UnresolvedInputError, CyclicDependencyError, OutputCollisionError } from './CellErrors'
import { UNKNOWN, BROKEN, FAILED, BLOCKED, WAITING, READY, RUNNING, OK, toInteger } from './CellStates'

const MSG_UNRESOLVED_INPUT = 'Unresolved input.'

export default class CellGraph {

  constructor() {
    // cell data by id
    this._cells = {}
    // symbols -> cell ids; which cell is depending on a symbol
    this._ins = {}
    // symbol -> cell id; which symbol is provided by which cell
    this._out = {}
    // to record which cells have been affected (batched update)
    // state changes are propagated through the graph in topological order
    // 'structureChanged': all cells which should be re-examined regarding
    // structural consistency.
    this._structureChanged = new Set()
    // 'valueUpdated': all cells that have a new value or a runtime error,
    // i.e. the new state is OK or FAILED.
    this._valueUpdated = new Set()
  }

  addCell(cell) {
    const id = cell.id
    if (this._cells[id]) throw new Error(`Cell with ${id} already exists`)
    this._cells[id] = cell
    this._structureChanged.add(id)

    if (cell.inputs.size > 0) {
      this._registerInputs(id, new Set(), cell.inputs)
    }
    if (cell.output) {
      this._registerOutput(id, null, cell.output)
    }
  }

  setInputs(id, newInputs) {
    let cell = this._cells[id]
    if (!cell) throw new Error(`Unknown cell ${id}`)
    newInputs = new Set(newInputs)
    if(this._registerInputs(cell.id, cell.inputs, newInputs)) {
      cell.inputs = newInputs
      this._clearCyclicDependencyError(cell)
      cell.clearErrors(e => e instanceof UnresolvedInputError)
    }
  }

  setOutput(id, newOutput) {
    // TODO: handle collisions
    // -> it would be nice if the graph could keep competing outputs and resolve
    //    automatically if all ambiguities have been resolved
    // TODO: if only the output of a cell changed, we could retain the runtime result
    // and leave the cell's state untouched
    let cell = this._cells[id]
    let oldOutput = cell.output
    if (this._registerOutput(id, oldOutput, newOutput)) {
      cell.output = newOutput
      // TODO: do we need to clear a potential old graph error
      // e.g. from a previous cyclic dependency
      this._clearCyclicDependencyError(cell)
    }
  }

  addError(id, error) {
    let cell = this._cells[id]
    cell.errors.push(error)
    this._structureChanged.add(id)
  }

  clearErrors(id, type) {
    let cell = this._cells[id]
    cell.clearErrors(type)
    this._structureChanged.add(id)
  }

  setResult(id, value, errors) {
    let cell = this._cells[id]
    cell.value = value
    // TODO: make sure that this is only set when we want it
    if (errors && errors.length > 0) {
      cell.state = FAILED
      cell.addErrors(errors)
    } else {
      cell.state = OK
    }
    this._valueUpdated.add(id)
  }

  _registerInputs(id, oldInputs, newInputs) {
    let toAdd = new Set(newInputs)
    let toRemove = new Set()

    if (oldInputs) {
      oldInputs.forEach(s => {
        if (newInputs.has(s)) {
          toAdd.delete(s)
        } else {
          toRemove.add(s)
        }
      })
    }
    toRemove.forEach(s => {
      // TODO: should this be made robust
      // actually it should not happen that the symbol is not registered yet
      this._ins[s].delete(id)
    })
    toAdd.forEach(s => {
      let ins = this._ins[s]
      if (!ins) ins = this._ins[s] = new Set()
      ins.add(id)
    })
    if (toAdd.size > 0 || toRemove.size > 0) {
      this._structureChanged.add(id)
      return true
    } else {
      return false
    }
  }

  _registerOutput(id, oldOutput, newOutput) {
    // nothing to be done if no change
    if (oldOutput === newOutput) return false
    // deregister the old output first
    if (oldOutput) {
      if (this._hasOutputCollision(oldOutput)) {
        this._resolveOutputCollision(oldOutput, id)
      } else {
        delete this._out[oldOutput]
        // mark old deps as affected
        let ids = this._ins[oldOutput] || []
        ids.forEach(_id => {
          let cell = this._cells[_id]
          if (cell.state === BROKEN) {
            // TODO: probably we do not want to clear all graph errors, but only specific ones
            cell.clearErrors('graph')
          }
          this._structureChanged.add(_id)
        })
      }
    }
    if (newOutput) {
      // TODO: detect collisions
      if (this._out[newOutput]) {
        let conflictingIds = this._out[newOutput]
        if (isString(conflictingIds)) conflictingIds = [conflictingIds]
        conflictingIds = new Set(conflictingIds)
        conflictingIds.add(id)
        conflictingIds = Array.from(conflictingIds)
        this._out[newOutput] = conflictingIds
        this._addOutputCollisionError(conflictingIds)
      } else {
        this._out[newOutput] = id
      }
      // mark new deps as affected
      let ids = this._ins[newOutput] || []
      ids.forEach(_id => {
        let cell = this._cells[_id]
        if (cell.state === BROKEN) {
          // TODO: probably we do not want to clear all graph errors, but only specific ones
          cell.clearErrors('graph')
        }
        this._structureChanged.add(_id)
      })
    }
    return true
  }

  update() {
    // a set of cell ids that have been updated
    let updated = new Set()

    // examine the graph structure
    // Note: we should not need to update the whole graph, still, we can do an
    // exhaustive update, because this is not performance critical
    let levels = {}
    this._structureChanged.forEach(id => {
      // detect unresolvable inputs
      this._detectUnresolvableInputs(id)
      // deterimine the dependency level and check for cyclic dependencies
      // Note: in case of a cyclic dependency we want to set all involved
      // cells into BROKEN state
      this._computeDependencyLevel(id, levels, updated)
      updated.add(id)
    })

    if (this._valueUpdated.size > 0) {
      this._valueUpdated.forEach(id => updated.add(id))
      // propagate state updates starting at cells after the cells that had a value update
      this._updateStates(this._getFollowSet(this._valueUpdated), updated)
    }

    if (this._structureChanged.size > 0) {
      // then propagate state updates for all structural changes
      this._updateStates(this._structureChanged, updated)
    }


    this._structureChanged.clear()
    this._valueUpdated.clear()

    return updated
  }

  _detectUnresolvableInputs(id) {
    let cell = this._cells[id]
    // detect unresolvable inputs
    let inputs = Array.from(cell.inputs)
    let unresolved = inputs.filter(s => !this._resolve(s))
    if (unresolved.length > 0) {
      // TODO: maybe only clear UnresolvedInputErrors
      cell.clearErrors('graph')
      cell.errors.push(new UnresolvedInputError(MSG_UNRESOLVED_INPUT, { unresolved }))
      cell.state = BROKEN
    }
  }

  _computeDependencyLevel(id, levels, updated, trace = new Set()) {
    let cell = this._cells[id]
    let inputs = Array.from(cell.inputs)
    trace = new Set(trace)
    trace.add(id)
    let inputLevels = inputs.map(s => {
      let inputId = this._resolve(s)
      if (!inputId) return 0
      if (trace.has(inputId)) {
        this._handleCycle(trace, updated)
        return Infinity
      }
      // do not recurse if the level has been computed already
      if (levels.hasOwnProperty(inputId)) {
        return levels[inputId]
      } else {
        return this._computeDependencyLevel(inputId, levels, updated, trace)
      }
    })
    let level = inputLevels.length > 0 ? Math.max(...inputLevels) + 1 : 0
    levels[id] = level
    cell.level = level
    return level
  }

  _getAffectedCellsSorted(ids) {
    let cells = []
    let visited = new Set()
    let q = Array.from(ids)
    while(q.length > 0) {
      let id = q.shift()
      if (visited.has(id)) continue
      visited.add(id)
      const cell = this._cells[id]
      const level = cell.level
      if (!cells[level]) cells[level] = []
      cells[level].push(cell)
      // process dependencies
      // ensuring that this cell is actually providing the output
      if (cell.output && cell.id === this._resolve(cell.output)) {
        let deps = Array.from(this._ins[cell.output] || [])
        q = q.concat(deps.filter(id => !visited[id]))
      }
    }
    return flatten(cells.filter(Boolean))
  }

  _updateStates(ids, updated) {
    // get all affected cells, i.e. all cells that are depending
    // on the cells with given ids
    let cells = this._getAffectedCellsSorted(ids)
    // determine the cell state from the state of their inputs
    cells.forEach(cell => this._updateCellState(cell, updated))
  }

  _updateCellState(cell, updated) {
    if (cell.hasError('engine') || cell.hasError('graph')) {
      if (cell.state === BROKEN) return
      cell.state = BROKEN
      updated.add(cell.id)
      return
    }
    if (cell.state === BROKEN) {
      cell.state = UNKNOWN
    }
    let inputs = Array.from(cell.inputs)
    if (inputs.length === 0) {
      cell.state = READY
      return
    }
    let inputStates = inputs.map(s => {
      let cell = this._cells[this._resolve(s)]
      if (cell) {
        // NOTE: for development we kept the less performant but more readable
        // representation as symbols, instead of ints
        return toInteger(cell.state)
      } else {
        return undefined
      }
    }).filter(Boolean)
    let inputState = Math.min(...inputStates)
    // Note: it is easier to do this in an arithmetic way
    // than in boolean logic
    let newState
    if (inputState <= toInteger(BLOCKED)) {
      newState = BLOCKED
    } else if (inputState <= toInteger(RUNNING)) {
      newState = WAITING
    } else { // if (inputState === OK) {
      newState = READY
    }
    if (newState && newState !== cell.state) {
      cell.state = newState
      updated.add(cell.id)
    }
  }

  _resolve(symbol) {
    let out = this._out[symbol]
    if (out) {
      if (isString(out)) return out
      else return out[0]
    }
  }

  // set of cell ids that depend on the given
  _getFollowSet(ids) {
    let followSet = new Set()
    ids.forEach(id => {
      const cell = this._cells[id]
      if (cell.output && id === this._resolve(cell.output)) {
        let followers = this._ins[cell.output]
        if (followers) {
          followers.forEach(id => followSet.add(id))
        }
      }
    })
    return followSet
  }

  _handleCycle(trace, updated) {
    let error = new CyclicDependencyError('Cyclic dependency', { trace })
    trace.forEach(id => {
      let cell = this._cells[id]
      cell.state = BROKEN
      cell.errors.push(error)
      cell.level = Infinity
      updated.add(id)
    })
  }

  _clearCyclicDependencyError(cell) {
    let err = cell.errors.find(err => err instanceof CyclicDependencyError)
    if (err) {
      const trace = err.trace
      trace.forEach(id => {
        let cell = this._cells[id]
        cell.errors = cell.errors.filter(err => !(err instanceof CyclicDependencyError))
        this._structureChanged.add(id)
      })
    }
  }

  _hasOutputCollision(symbol) {
    return isArray(this._out[symbol])
  }

  _addOutputCollisionError(ids) {
    let err = new OutputCollisionError('Competing output declarations.', { ids })
    ids.forEach(id => {
      let cell = this._cells[id]
      cell.clearErrors(e => e instanceof OutputCollisionError)
      cell.errors.push(err)
      this._structureChanged.add(id)
    })
  }

  _removeOutputCollisionError(id) {
    let cell = this._cells[id]
    cell.clearErrors(e => e instanceof OutputCollisionError)
    this._structureChanged.add(id)
  }

  /*
    called whenever an output variable is changed
    Removes the cell id from the list of competing cells
    and removes errors if possible.
  */
  _resolveOutputCollision(symbol, id) {
    let out = this._out[symbol]
    // in case of collisions we store the competing cell ids as array
    if (isArray(out)) {
      this._removeOutputCollisionError(id)
      let s = new Set(out)
      s.delete(id)
      s = Array.from(s)
      if (s.length > 1) {
        this._out[symbol] = s
      } else {
        let _id = s[0]
        this._out[symbol] = _id
        this._removeOutputCollisionError(_id)
      }
    }
  }
}
