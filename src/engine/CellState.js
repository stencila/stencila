import { EventEmitter } from 'substance'

const INITIAL = Symbol('initial')
const ANALYSED = Symbol('analysed')
const EVALUATED = Symbol('evaluated')

const TRANSITIONS = {
  INITIAL: { ANALYSED },
  ANALYSED: { INITIAL, EVALUATED },
  EVALUATED: { INITIAL, ANALYSED }
}

/*
  Managed by Engine.
*/
export default class CellState extends EventEmitter {

  constructor() {
    super()

    this.state = INITIAL

    this.inputs = []
    this.output = null

    this.messages = []
    this.value = null
  }

  transitionTo(newState) {
    const oldState = this.state
    const T = TRANSITIONS[oldState]
    if (!T) throw new Error(`Invalid State: ${oldState}`)
    if (!T[newState]) throw new Error(`Invalid Transition: ${oldState} -> ${newState}`)
    if (this.onchange) {
      this.onchange(newState, oldState)
    } else {
      this.emit('change', newState, oldState)
    }
  }

  hasErrors() {
    // TODO
    return false
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

export { Cell, INITIAL, ANALYSED, EVALUATED }
