import NodeComponent from 'substance/ui/NodeComponent'

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
