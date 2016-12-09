import InlineNode from 'substance/model/InlineNode'

class Input extends InlineNode {

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
    this.document.setVariable(this.name, {
      type: this.dataType,
      format: 'str',
      value: value
    })
  }

}

Input.define({
  type: 'input',
  name: { type: 'string' },
  displayType: { type: 'string', default: 'text' },
  dataType: { type: 'string', default: 'text' }
})

export default Input

