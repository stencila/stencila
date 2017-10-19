import { EventEmitter } from 'substance'

const INITIAL = Symbol('initial')
const ANALYSED = Symbol('analysed')
const EVALUATED = Symbol('evaluated')

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

  hasErrors() {
    return this.messages && this.messages.length > 0
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

export { CellState, INITIAL, ANALYSED, EVALUATED }
