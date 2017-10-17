import { Component } from 'substance'

export default
class NumberValueComponent extends Component {
  render($$) {
    let value = this.props.value
    let el = $$('div').addClass('sc-number-value')
    el.append(value.data)
    return el
  }
}
