import { Component } from 'substance'

export default
class ImageValueComponent extends Component {
  render($$) {
    let value = this.props.value
    let el = $$('img')
      .attr('src', value.src)
      .addClass('sc-image-value')
    return el
  }
}
