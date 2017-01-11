import InlineNode from 'substance/model/InlineNode'

/**
 * For a full list of HTML5 input types see https://developer.mozilla.org/en/docs/Web/HTML/Element/input
 */
class Input extends InlineNode {

  /**
   * Convert the input into a data pack
   *
   * @return {Object} A data pack
   */
  getPack () {
    // Convert the input type into a data type
    let type = {
      'checkbox': 'bool',
      'number': 'flt',
      'range': 'flt',
      'text': 'str'
    }[this.type_] || 'str'
    // Return the data pack
    return {
      type: type,
      format: 'text',
      value: this.value
    }
  }

  /**
   * Set the value of this input
   *
   * Changes to the value an input are considered to be local and so
   * this does not trigger a document transaction. Instead, via `document.setVariable`
   * it triggers a refresh of `Execute` nodes.
   *
   * @param {String} value Value of input
   */
  setValue (value) {
    this.value = value
    this.document.setVariable(this.name, this.getPack())
  }

}

Input.define({
  type: 'input',

  name: { type: 'string' },
  type_: { type: 'string', default: 'text' },
  value: {type: 'string', optional: true}
})

export default Input

