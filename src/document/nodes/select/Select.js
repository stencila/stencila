import InlineNode from 'substance/model/InlineNode'

class Select extends InlineNode {

  /**
   * Convert the select into a data pack
   *
   * @return {Object} A data pack
   */
  getPack () {
    return {
      type: 'str',
      format: 'text',
      value: this.value
    }
  }

  /**
   * Set the value of this select
   *
   * @param {String} value Value of select
   */
  setValue (value) {
    this.value = value
    this.document.setVariable(this.name, this.getPack())
  }

}

Select.define({
  type: 'select',

  name: { type: 'string' },
  options: { type: ['array', 'string'], default: [] },
  value: { type: 'string', optional: true }
})

export default Select

