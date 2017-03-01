import { NodeComponent } from 'substance'

class ImageComponent extends NodeComponent {

  render ($$) {
    let node = this.props.node
    return super.render($$)
      .addClass('sc-image')
      .append(
        $$('img').attr({
          src: node.src
        })
      )
  }

}

export default ImageComponent
