import IsolatedNodeComponent from 'substance/packages/isolated-node/IsolatedNodeComponent'
import ContainerEditor from 'substance/ui/ContainerEditor'

class DiscussionMarkdownComponent extends IsolatedNodeComponent {

  constructor (...args) {
    super(...args)

    this.ContentClass = ContainerEditor
  }

  render ($$) {
    return super.render.call(this, $$)
      .addClass('sc-discussion')
  }
}

export default DiscussionMarkdownComponent
