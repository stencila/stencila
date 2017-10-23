import { EventEmitter } from 'substance'

const INITIAL = Symbol('initial')
const ANALYSED = Symbol('analysed')
const EVALUATED = Symbol('evaluated')

// some of the inputs are not ready yet
const PENDING = Symbol('pending')
// some of the inputs has errored
const INPUT_ERROR = Symbol('input-error')
// all inputs are ready and this cell will be executed ASAP
const INPUT_READY = Symbol('input-ready')
// cell is being evluated
const RUNNING = Symbol('running')
// cell has been evaluated but failed
const ERROR = Symbol('error')
// cell has successfully been evaluated
const OK = Symbol('ok')

export default class CellState extends EventEmitter {

  constructor() {
    super()

    this.status = PENDING
    this._engineState = INITIAL

    this.inputs = []
    this.output = null

    this.messages = []
    this.value = null
  }

  hasErrors() {
    return hasError(this)
  }

  hasOutput() {
    return Boolean(this.output)
  }

  hasValue() {
    return Boolean(this.value)
  }

  getValue() {
    return this.value
  }

}

function deriveCellStatus(cellState) {
  switch(cellState._engineState) {
    case INITIAL: {
      if (hasError(cellState)) {
        cellState.status = ERROR
      } else {
        cellState.status = PENDING
      }
      break
    }
    case ANALYSED: {
      if (hasError(cellState)) {
        cellState.status = ERROR
      } else {
        cellState.status = PENDING
      }
      break
    }
    case EVALUATED: {
      if (hasError(cellState)) {
        cellState.status = ERROR
      } else {
        cellState.status = OK
      }
      break
    }
    default:
      // should not happen
      throw new Error('Invalid state')
  }
}

function hasError(cellState) {
  if (cellState.messages) {
    let messages = cellState.messages
    for(let i = 0; i < messages.length; i++) {
      if (messages[i].type === 'error'
        // HACK: messages should have the right format
        // but we want to get things running
        || messages[i] instanceof Error) {
        return true
      }
    }
  }
  return false
}

export { CellState,
  INITIAL, ANALYSED, EVALUATED,
  PENDING, INPUT_ERROR, INPUT_READY,
  RUNNING, ERROR, OK,
  deriveCellStatus, hasError
}
