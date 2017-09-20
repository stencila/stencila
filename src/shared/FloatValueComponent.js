import { Component } from 'substance'

export default
class FloatValueComponent extends Component {
  render($$) {
    let el = $$('div').addClass('sc-float-value')
    el.append(this.props.value)
    return el
  }
}
