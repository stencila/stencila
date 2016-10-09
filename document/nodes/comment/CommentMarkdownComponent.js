import IsolatedNodeComponent from 'substance/packages/isolated-node/IsolatedNodeComponent'

import CommentComponent from './CommentComponent'

class CommentMarkdownComponent extends CommentComponent {

  render ($$) {
    var node = this.props.node
    return IsolatedNodeComponent.prototype.render.call(this, $$)
      .insertAt(0,
        $$('div')
          .ref('header')
          .addClass('se-header')
          .attr('contenteditable', false)
          .append(
            $$('span')
              .ref('who')
              .addClass('se-who')
              .text(node.who),
            $$('span')
              .ref('when')
              .addClass('se-when')
              .text(node.when)
          )
      )
  }

}

export default CommentMarkdownComponent
