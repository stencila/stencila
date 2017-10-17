import { Component } from 'substance'

export default
class ArrayValueComponent extends Component {
  render($$) {
    let value = this.props.value
    let el = $$('div').addClass('sc-array-value')
    value.data.forEach((item) => {
      el.append(
        $$('div').addClass('se-array-item').append(item.toString())
      )
    })
    return el
  }
}
