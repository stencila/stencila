import {IsolatedNodeComponent, ContainerEditor} from 'substance'

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
