import { InlineNode } from 'substance'

export default
class InlineInputNode extends InlineNode {

  get value() {
    return this._value
  }

  set value(value) {
    this._value = value
    this.emit('value:updated', this)
  }

  get name() {
    return this._name
  }

  set name(name) {
    const oldName = this._name
    this._name = name
    this.emit('name:updated', this, oldName)
  }

}
