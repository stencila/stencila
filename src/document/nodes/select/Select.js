import InlineInputNode from '../InlineInputNode'

class Select extends InlineInputNode {

  get selectedIndex() {
    return this._selectedIndex
  }

  set selectedIndex(selectedIndex) {
    this._selectedIndex = selectedIndex
    const option = this.options[selectedIndex]
    this.value = option ? option.value : undefined
  }

  get text() {
    const option = this.options[this._selectedIndex]
    return option ? option.text : ''
  }

}

Select.schema = {
  type: 'select',
  name: { type: 'string' },
  options: { type: ['array', 'object'], default: [] },
  selectedIndex: { type: 'number', optional: true }
}

export default Select
