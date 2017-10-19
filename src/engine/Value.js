import { EventEmitter } from 'substance'

const INITIAL = Symbol('initial')
const PENDING = Symbol('pending')
const READY = Symbol('ready')
const ERROR = Symbol('error')

const TRANSITIONS = {
  INITIAL: { PENDING },
  PENDING: { INITIAL, READY, ERROR },
  READY: { PENDING },
  ERROR: { PENDING }
}

/*
  Managed by Engine.
*/
export default class Value extends EventEmitter {

  constructor() {
    super()

    this.state = INITIAL
  }

  transitionTo(newState) {
    const oldState = this.state
    const T = TRANSITIONS[oldState]
    if (!T) throw new Error(`Invalid State: ${oldState}`)
    if (!T[newState]) throw new Error(`Invalid Transition: ${oldState} -> ${newState}`)
    this.emit('state:changed', newState, oldState)
  }

}

export { Value, INITIAL, PENDING, READY, ERROR }
