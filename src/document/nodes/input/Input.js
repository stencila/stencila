import { InlineNode } from 'substance'

/**
 * For a full list of HTML5 input types see https://developer.mozilla.org/en/docs/Web/HTML/Element/input
 */
class Input extends InlineNode {

  /**
   * Set the value of this input
   *
   * Changes to the value an input are considered to be local and so
   * this does not trigger a document transaction. Instead, via `document.setVariable`
   * it triggers a refresh of `Execute` nodes.
   *
   * @param {string} string String value of input
   */
  setValue (string) {
    this.value = string
    this.document.setVariable(this.name, this.getValue())
  }

  getValue () {
    // Convert to a Javascript value
    /*
    TODO conversion from more types
    TODO validation

      'checkbox': 'bool',
      'number': 'flt',
      'range': 'flt',
      'string': 'str',
      'text': 'str'
    */
    if (!this.value) return null
    if (this.type_ === 'json') return JSON.parse(this.value)
    return this.value
  }
}

Input.define({
  type: 'input',

  name: { type: 'string' },
  type_: { type: 'string', default: 'text' },
  value: {type: 'string', optional: true}
})

export default Input

