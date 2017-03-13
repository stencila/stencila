import InlineInputNode from '../InlineInputNode'

/**
 * For a full list of HTML5 input types see https://developer.mozilla.org/en/docs/Web/HTML/Element/input
 */
class RangeInput extends InlineInputNode {

  get inputType() {
    return 'range'
  }

}

RangeInput.schema = {
  type: 'range-input',
  name: { type: 'string' },
  value: { type: 'number', optional: true },
  min: { type: 'number', optional: true },
  max: { type: 'number', optional: true },
  step: { type: 'number', optional: true }
}

export default RangeInput
