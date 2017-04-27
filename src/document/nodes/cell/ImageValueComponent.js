import { Component } from 'substance'

export default
class ImageValueComponent extends Component {

  render($$) {
    let el = $$('img').addClass('sc-image-value')
    return el
  }

  didUpdate() {
    // TODO: `didUpdate()` is not the right place for this, where is?
    this.el.getNativeElement().src = URL.createObjectURL(this.props.value.blob)
  }

}
