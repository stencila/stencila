import { Component } from 'substance'

export default
class ImageValueComponent extends Component {

  render($$) {
    let el = $$('img').addClass('sc-image-value')
    return el
  }

  didUpdate() {
    // TODO: `didUpdate()` is not the right place for this, where is?
    let image = this.props.value
    let src
    try {
      src = window.URL.createObjectURL(image.blob)
    } catch (error) {
      src = `data:image/${image.format};base64,${image.base64}`
    }
    this.el.getNativeElement().src = src
  }

}
