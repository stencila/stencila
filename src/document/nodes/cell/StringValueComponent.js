import { Component } from 'substance'

export default
class StringValueComponent extends Component {
  render($$) {
    let el = $$('div').addClass('sc-string-value')
    el.append(
      "'",
      this.props.value,
      "'"
    )
    return el
  }
}
