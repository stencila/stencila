import { Component } from 'substance'

export default
class FloatValueComponent extends Component {
  render($$) {
    let cell = this.props.cell
    let el = $$('div').addClass('sc-float-value')
    if (cell.hasErrors()) {
      el.addClass('sm-has-errors')
    }
    el.append(this.props.value)
    return el
  }
}
