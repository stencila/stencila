import { uuid } from 'substance'

// this key is used to store data on the node instance
const KEY = uuid()

class CellAdapter {

  constructor(cell) {
    this.node = cell
  }

  emit(...args) {
    this.node.emit(...args)
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
