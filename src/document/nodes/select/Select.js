import InlineNode from 'substance/model/InlineNode'

class Select extends InlineNode {
  getSelectedText() {
    return this.options[this.selectedIndex].text
  }

  getValue() {
    return this.options[this.selectedIndex].value
  }
}

Select.define({
  type: 'select',
  name: { type: 'string' },
  options: { type: ['array', 'object'], default: [] },
  selectedIndex: { type: 'number', optional: true }
})

export default Select
