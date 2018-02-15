import { Component } from 'substance'

export default
class BooleanrValueComponent extends Component {
  render($$) {
    let value = this.props.value
    let el = $$('div').addClass('sc-boolean-value')
    el.append(value.data ? 'true' : 'false')
    return el
  }
}
