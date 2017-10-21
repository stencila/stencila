import { uuid } from 'substance'
import CellState from '../engine/CellState'

// this key is used to store data on the node instance
const KEY = uuid()

export function getCellAdapter(cell) {
  let adapter = cell[KEY]
  if (!adapter) {
    cell[KEY] = adapter = new CellAdapter(cell)
  }
  return adapter
}

export function getInputAdapter(input) {
  let adapter = input[KEY]
  if (!adapter) {
    input[KEY] = adapter = new InputAdapter(input)
  }
  return adapter
}

export function getCellState(cell) {
  return cell.state
}

class CellAdapter {

  constructor(cell) {
    this.node = cell

    // TODO: would be good to have an API for that
    this.state = new CellState()
    cell.state = this.state
  }

  emit(...args) {
    this.node.emit(...args)
  }

  isCell() {
    return true
  }

  isInput() {
    return false
  }

  get id() {
    return this.node.id
  }

  get source() {
    const sourceEl = this._getSourceElement()
    return sourceEl.textContent
  }

  get language() {
    const sourceEl = this._getSourceElement()
    return sourceEl.getAttribute('language')
  }

  get inputs() {
    return this.state.inputs
  }

  get output() {
    return this.state.output
  }

  _getSourceElement() {
    if (!this._source) {
      this._source = this.node.find('source-code')
    }
    return this._source
  }

}

class InputAdapter {

  constructor(input) {
    this.node = input
  }

  emit(...args) {
    this.node.emit(...args)
  }

  isInput() {
    return true
  }

  get id() {
    return this.node.id
  }

  get name() {
    return this.node.getAttribute('name')
  }

  get value() {
    return this.node.getAttribute('value')
  }

}
