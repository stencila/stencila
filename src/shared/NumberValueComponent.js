import { Component } from 'substance'

export default
class NumberValueComponent extends Component {
  render($$) {
    let value = this.props.value
    let el = $$('div').addClass('sc-number-value')
    // TODO: Better formatting of numbers (not always 6 digits)
    el.append(value.data.toFixed(6))
    return el
  }
}
