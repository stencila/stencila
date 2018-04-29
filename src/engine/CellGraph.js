import { flatten, isString, isArray } from 'substance'
import { CellError, UnresolvedInputError, CyclicDependencyError, OutputCollisionError } from './CellErrors'
import { UNKNOWN, ANALYSED, BROKEN, FAILED, BLOCKED, WAITING, READY, RUNNING, OK, toInteger } from './CellStates'

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
    // - 'stateChanged': an error has been added or the state has been updated otherwise
    // - 'structureChanged': cells that have changed w.r.t. inputs or output, and thus need to be checked for structural consistency
    // - 'valueUpdated': all cells that have a new value or a runtime error, i.e. the new state is OK or FAILED.
    // TODO: shouldn't be 'valueUpdated' just be the same as 'stateChanged'?
    this._stateChanged = new Set()
    this._structureChanged = new Set()
    this._valueUpdated = new Set()
  }

  needsUpdate() {
    return this._stateChanged.size > 0 || this._structureChanged.size > 0 || this._valueUpdated.size > 0
  }

  hasCell(id) {
    return this._cells.hasOwnProperty(id)
  }

  getCell(id) {
    return this._cells[id]
  }

  addCell(cell) {
    const id = cell.id
    if (this._cells[id]) throw new Error(`Cell with ${id} already exists`)
    this._cells[id] = cell
    this._structureChanged.add(id)
    if (cell.inputs && cell.inputs.size > 0) {
      this._registerInputs(id, new Set(), cell.inputs)
    }
    if (cell.output) {
      this._registerOutput(id, null, cell.output)
    }
  }

  removeCell(id) {
    const cell = this._cells[id]
    if (!cell) throw new Error('Internal error: cell does not exist.')
    cell.inputs.forEach(s => {
      if (this._ins[s]) {
        this._ins[s].delete(id)
      }
    })
    this._deregisterOutput(id, cell.output)
    delete this._cells[id]
    if (cell.prev) {
      this._setNext(cell.prev, cell.next)
    }
    if (cell.next) {
      this._setPrev(cell.next, cell.prev)
    }
    // remove the cell from all registered updates
    this._stateChanged.delete(cell.id)
    this._structureChanged.delete(cell.id)
    this._valueUpdated.delete(cell.id)
  }

  getValue(symbol) {
    let cellId = this._out[symbol]
    if (!cellId) return undefined
    // if there is a name collision return undefined
    // TODO: should we allow this at all?
    if (isArray(cellId)) {
      throw new Error('Ambigous symbol: '+symbol)
    }

    const cell = this._cells[cellId]
    if (!cell) throw new Error('Internal error: cell does not exist.')
    // Note, that the cell value is actually not interpreted in any way by the graph
    // it is maintained by the engine.
    return cell.value
  }

  setInputsOutputs(id, newInputs, newOutput) {
    let cell = this._cells[id]
    if (!cell) throw new Error(`Unknown cell ${id}`)
    this._setInputs(cell, newInputs)
    this._setOutput(cell, newOutput)
    if (cell.status === UNKNOWN) {
      this._structureChanged.add(id)
      cell.status = ANALYSED
    }
  }

  // Note: we use this for sheet cells
  setInputs(id, newInputs) {
    let cell = this._cells[id]
    if (!cell) throw new Error(`Unknown cell ${id}`)
    this._setInputs(cell, newInputs)
    if (cell.status === UNKNOWN) {
      this._structureChanged.add(id)
      cell.status = ANALYSED
    }
  }

  // used to update sheet cell output symbols after structural
  // changes. In this case there are typically a lot of other changes, too
  setOutput(id, newOutput) {
    let cell = this._cells[id]
    if (!cell) throw new Error(`Unknown cell ${id}`)
    this._setOutput(cell, newOutput)
  }

  _setInputs(cell, newInputs) {
    newInputs = new Set(newInputs)
    if(this._registerInputs(cell.id, cell.inputs, newInputs)) {
      cell.inputs = newInputs
      this._clearCyclicDependencyError(cell)
      cell.clearErrors(e => e instanceof UnresolvedInputError)
    }
  }

  _setOutput(cell, newOutput) {
    // TODO: if only the output of a cell changed, we could retain the runtime result
    // and leave the cell's state untouched
    let oldOutput = cell.output
    if (this._registerOutput(cell.id, oldOutput, newOutput)) {
      cell.output = newOutput
      // TODO: do we need to clear a potential old graph error
      // e.g. from a previous cyclic dependency
      this._clearCyclicDependencyError(cell)
    }
  }

  _setNext(id, nextId) {
    let cell = this._cells[id]
    cell.next = nextId
    this._structureChanged.add(nextId)
  }

  _setPrev(id, prevId) {
    let cell = this._cells[id]
    cell.prev = prevId
    this._structureChanged.add(id)
  }

  addError(id, error) {
    this.addErrors(id, [error])
  }

  addErrors(id, errors) {
    let cell = this._cells[id]
    errors = errors.map(err => CellError.cast(err))
    cell.addErrors(errors)
    this._stateChanged.add(id)
  }

  clearErrors(id, type) {
    let cell = this._cells[id]
    cell.clearErrors(type)
    this._stateChanged.add(id)
  }

  setValue(id, value) {
    let cell = this._cells[id]
    cell.value = value
    if (cell.hasErrors()) {
      cell.status = FAILED
    } else {
      cell.status = OK
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
      if (this._ins[s]) {
        this._ins[s].delete(id)
      }
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
      this._deregisterOutput(id, oldOutput)
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
        // consider every related id for re-examination
        conflictingIds.forEach(_id => this._structureChanged.add(_id))
      } else {
        this._out[newOutput] = id
      }
      // mark new deps as affected
      let ids = this._ins[newOutput] || []
      ids.forEach(_id => {
        let cell = this._cells[_id]
        if (cell.status === BROKEN) {
          // TODO: probably we do not want to clear all graph errors, but only specific ones
          cell.clearErrors('graph')
        }
        this._structureChanged.add(_id)
      })
    }
    return true
  }

  _deregisterOutput(id, output) {
    if (this._hasOutputCollision(output)) {
      this._resolveOutputCollision(output, id)
    } else {
      delete this._out[output]
      // mark old deps as affected
      let ids = this._ins[output] || []
      ids.forEach(_id => {
        let cell = this._cells[_id]
        if (cell.status === BROKEN) {
          // TODO: probably we do not want to clear all graph errors, but only specific ones
          cell.clearErrors('graph')
        }
        this._structureChanged.add(_id)
      })
    }
  }

  update() {
    // a set of cell ids that have been updated
    let updated = new Set()
    let stateChanged = this._stateChanged

    // examine the graph structure
    // Note: we should not need to update the whole graph, still, we can do an
    // exhaustive update, because this is not performance critical
    let levels = {}
    this._structureChanged.forEach(id => {
      // detect unresolvable inputs
      this._detectUnresolvableInputs(id)
      // detect output collisions
      this._detectOutputCollisions(id)
      // deterimine the dependency level and check for cyclic dependencies
      // Note: in case of a cyclic dependency we want to set all involved
      // cells into BROKEN state
      this._computeDependencyLevel(id, levels, updated)
      updated.add(id)
      // mark cells with structure changes for state update
      stateChanged.add(id)
    })

    if (this._valueUpdated.size > 0) {
      this._valueUpdated.forEach(id => {
        updated.add(id)
      })
      // mark all followers for a state update
      this._getFollowSet(this._valueUpdated).forEach(id => {
        let cell = this._cells[id]
        cell.clearErrors('runtime')
        stateChanged.add(id)
      })
    }

    if (stateChanged.size > 0) {
      // then propagate state updates for all structural changes
      this._updateStates(stateChanged, updated)
    }

    this._stateChanged.clear()
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
      cell.addErrors([new UnresolvedInputError(MSG_UNRESOLVED_INPUT, { unresolved })])
      cell.status = BROKEN
    }
  }

  _detectOutputCollisions(id) {
    let cell = this._cells[id]
    let output = cell.output
    if (!output) return
    let ids = this._out[output]
    if (isArray(ids)) {
      // TODO: is there a more efficient way?
      cell.clearErrors(e => e instanceof OutputCollisionError)
      cell.addErrors([new OutputCollisionError('Competing output declarations.', { ids })])
    }
  }

  _computeDependencyLevel(id, levels, updated, trace = new Set()) {
    let cell = this._cells[id]
    let inputs = Array.from(cell.inputs)
    trace = new Set(trace)
    trace.add(id)

    const _recursive = (id) => {
      if (trace.has(id)) {
        this._handleCycle(trace, updated)
        return Infinity
      }
      if (levels.hasOwnProperty(id)) {
        return levels[id]
      } else {
        return this._computeDependencyLevel(id, levels, updated, trace)
      }
    }

    let inputLevels = []
    inputs.forEach(s => {
      let res = this._resolve(s)
      if (!res) return 0
      if (isString(res)) {
        inputLevels.push(_recursive(res))
      } else {
        res.forEach(id => {
          inputLevels.push(_recursive(id))
        })
      }
    })
    // EXPERIMENTAL: considering an explicitly set predecessor to preserve natural order where appropriate
    if (cell.prev) {
      inputLevels.push(_recursive(cell.prev))
    }
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
      let affected = this._getAffected(cell)
      q = q.concat(affected.filter(id => !visited[id]))
    }
    // Note: remove bins for levels that are not affected
    return flatten(cells.filter(Boolean))
  }

  _getAffected(cell) {
    let affected = []
    if (this._cellProvidesOutput(cell)) {
      affected = Array.from(this._ins[cell.output] || [])
    }
    if (cell.hasSideEffects && cell.next) {
      // find next cell with side effects
      for (let nextId = cell.next; nextId; nextId = cell.next) {
        let nextCell = this._cells[nextId]
        if (nextCell && nextCell.hasSideEffects) {
          affected.push(nextId)
          break
        }
      }
    }
    return affected
  }

  _updateStates(ids, updated) {
    // get all affected cells, i.e. all cells that are depending
    // on the cells with given ids
    let cells = this._getAffectedCellsSorted(ids)
    // determine the cell state from the state of their inputs
    cells.forEach(cell => this._updateCellState(cell, updated))
  }

  _updateCellState(cell, updated) {
    // invariant detection of BROKEN state
    if (cell.hasError('engine') || cell.hasError('graph')) {
      if (cell.status === BROKEN) return
      cell.status = BROKEN
      updated.add(cell.id)
      return
    }
    // skip cells which have not been registered yet
    if (cell.status === UNKNOWN) return
    // invariant detection of FAILED state
    if (cell.hasErrors()) {
      if (cell.status === FAILED) return
      cell.status = FAILED
      updated.add(cell.id)
      return
    }
    let inputs = Array.from(cell.inputs)
    if (!cell.hasSideEffects && inputs.length === 0) {
      cell.status = READY
      return
    }
    let inputStates = []
    inputs.forEach(s => {
      let res = this._resolve(s)
      if (!res) return
      if (isString(res)) {
        let _cell = this._cells[res]
        // NOTE: for development we kept the less performant but more readable
        // representation as symbols, instead of ints
        inputStates.push(toInteger(_cell.status))
      } else {
        res.forEach(id => {
          let _cell = this._cells[id]
          inputStates.push(toInteger(_cell.status))
        })
      }
    })
    if (cell.hasSideEffects && cell.prev) {
      let _cell = this._cells[cell.prev]
      if (_cell) {
        inputStates.push(toInteger(_cell.status))
      }
    }
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
    if (newState && newState !== cell.status) {
      cell.status = newState
      updated.add(cell.id)
    }
  }

  _resolve(symbol) {
    return this._out[symbol]
  }

  _cellProvidesOutput(cell) {
    return (cell.output && cell.id === this._out[cell.output])
  }

  // set of cell ids that depend on the given ones
  _getFollowSet(ids) {
    let followSet = new Set()
    ids.forEach(id => {
      const cell = this._cells[id]
      this._getAffected(cell).forEach(id => followSet.add(id))
    })
    return followSet
  }

  // get a set of all ids a cell is depending on (recursively)
  _getPredecessorSet(id, set) {
    if (!set) set = new Set()
    const _recursive = (id) => {
      if (!set.has(id)) {
        set.add(id)
        this._getPredecessorSet(id, set)
      }
    }

    let cell = this.getCell(id)
    cell.inputs.forEach(s => {
      let res = this._resolve(s)
      if (!res) return
      if (isString(res)) {
        _recursive(res)
      } else {
        res.forEach(_recursive)
      }
    })
    if (cell.hasSideEffects && cell.prev) {
      _recursive(cell.prev)
    }
    return set
  }

  _handleCycle(trace, updated) {
    let error = new CyclicDependencyError('Cyclic dependency', { trace })
    trace.forEach(id => {
      let cell = this._cells[id]
      cell.status = BROKEN
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
        cell.clearErrors(err => (err instanceof CyclicDependencyError))
        this._structureChanged.add(id)
      })
    }
  }

  _hasOutputCollision(symbol) {
    return isArray(this._out[symbol])
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
