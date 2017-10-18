import { Component } from 'substance'

export default
class IntegerValueComponent extends Component {
  render($$) {
    let value = this.props.value
    let el = $$('div').addClass('sc-integer-value')
    el.append(value.data)
    return el
  }
}
