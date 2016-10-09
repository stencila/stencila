import HeadingComponent from './HeadingComponent'

class HeadingMarkdownComponent extends HeadingComponent {
  render ($$) {
    var node = this.props.node
    return super.render.call(this, $$)
      .insertAt(0,
        $$('span')
          .ref('level')
          .text(Array(node.level + 1).join('#') + ' ')
      )
  }
}

export default HeadingMarkdownComponent
