import IsolatedNodeComponent from 'substance/packages/isolated-node/IsolatedNodeComponent'
import ContainerEditor from 'substance/ui/ContainerEditor'

import moment from 'moment'

class CommentComponent extends IsolatedNodeComponent {

  constructor (...args) {
    super(...args)

    this.ContentClass = ContainerEditor
  }

  /**
   * Method override for custom class names
   */
  getClassNames () {
    return 'sc-comment'
  }

  /**
   * Method override to disable the comment unless the current user
   * is the original author of the comment
   */
  isDisabled () {
    var user = this.context.editorSession.config.user
    return this.props.node.who !== ('@' + user)
  }

  /**
   * Method override so no blocker is rendered over this
   * `IsolatedNodeComponent` (requires two clicks to begin editing)
   */
  shouldRenderBlocker () {
    // CHECK Is this method needed?
    return false
  }

  /**
   * Method ovveride to add additional elements
   */
  render ($$) {
    var node = this.props.node
    return super.render.call(this, $$)
      .insertAt(0,
        $$('div')
          .ref('header')
          .addClass('se-header')
          .attr('contenteditable', false)
          .append(
            $$('div')
              .ref('who')
              .addClass('se-who')
              .text(node.who),
            $$('div')
              .ref('when')
              .addClass('se-when')
              .text(moment(node.when).fromNow())
          )
      )
  }
}

export default CommentComponent
