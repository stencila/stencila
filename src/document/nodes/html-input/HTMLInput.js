import InlineNode from 'substance/model/InlineNode'

/**
 * For a full list of HTML5 input types see https://developer.mozilla.org/en/docs/Web/HTML/Element/input
 */
class HTMLInput extends InlineNode {
  getValue() {
    return this.value
  }
}

HTMLInput.schema = {
  type: 'html-input',
  name: { type: 'string' },
  // Represents the HTML 5 input type (e.g. 'range' or 'number')
  inputType: { type: 'string', default: '' },
  min: { type: 'number', optional: true },
  max: { type: 'number', optional: true },
  step: { type: 'number', optional: true },
  value: { type: 'number', optional: true }
}

export default HTMLInput
