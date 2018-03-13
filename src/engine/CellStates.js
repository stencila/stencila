// not yet analysed
const UNKNOWN = Symbol('UNKNOWN')
// analysed
const ANALYSED = Symbol('ANALYSED')
// syntax or graph errors
const BROKEN = Symbol('BROKEN')
// runtime or validation errors
const FAILED = Symbol('FAILED')
// one of the inputs is broken, failed, or blocked
const BLOCKED = Symbol('BLOCKED')
// all inputs are ready, running, or ok
const WAITING = Symbol('WAITING')
// all inputs are ok
const READY = Symbol('READY')
// evaluation is running (READY+evaluation triggered)
// TODO: do we really need this on this level?
const RUNNING = Symbol('RUNNING')
// evaluation succeeded
const OK = Symbol('OK')

function toInteger(state) {
  switch (state) {
    case UNKNOWN: return -2
    case ANALYSED: return -1
    case BROKEN: return 1
    case FAILED: return 2
    case BLOCKED: return 3
    case WAITING: return 4
    case READY: return 5
    case RUNNING: return 6
    case OK: return 7
    default:
      throw new Error('Illegal state.')
  }
}

function toString(state) {
  switch (state) {
    case UNKNOWN: return 'unknown'
    case ANALYSED: return 'analysed'
    case BROKEN: return 'broken'
    case FAILED: return 'failed'
    case BLOCKED: return 'blocked'
    case WAITING: return 'waiting'
    case READY: return 'ready'
    case RUNNING: return 'running'
    case OK: return 'ok'
    default:
      throw new Error('Illegal state.')
  }
}

export { UNKNOWN, ANALYSED, BROKEN, FAILED, BLOCKED, WAITING, READY, RUNNING, OK, toInteger, toString }