import { Component } from 'substance'

export default
class IntegerValueComponent extends Component {
  render($$) {
    let el = $$('div').addClass('sc-integer-value')
    el.append(this.props.value)
    return el
  }
}
