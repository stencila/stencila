import { InlineNode } from 'substance'

class Select extends InlineNode {

  /**
   * Set the value of this select
   *
   * @param {String} value Value of select
   */
  setValue (value) {
    this.value = value
    this.document.setVariable(this.name, this.getValue())
  }

  /**
   * Convert the select into a Javascript value
   *
   * @return {Object} A data pack
   */
  getValue () {
    return this.value
  }

}

Select.define({
  type: 'select',

  name: { type: 'string' },
  options: { type: ['array', 'string'], default: [] },
  value: { type: 'string', optional: true }
})

export default Select

