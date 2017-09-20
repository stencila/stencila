import { Component } from 'substance'

export default
class ArrayValueComponent extends Component {
  render($$) {
    let el = $$('div').addClass('sc-array-value')
    let array = this.props.value

    array.forEach((item) => {
      el.append(
        $$('div').addClass('se-array-item').append(item.toString())
      )
    })
    return el
  }
}
