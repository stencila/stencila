import { AnnotationComponent, getRelativeBoundingRect } from 'substance'

class MarkComponent extends AnnotationComponent {

  render ($$) {
    var el = super.render.call(this, $$)
    el.on('click', this._selected, this)
    return el
  }

  /**
   * When a mark is selected notify the associated `DiscussionComponent`
   * to show itself
   */
  _selected () {
    // CHECK
    // Is there a better way to do this rather than having a
    // document based event?
    var position = getRelativeBoundingRect(
      this.el.el,
      this.context.scrollPane.refs.content.el.el
    )
    document.dispatchEvent(new window.CustomEvent('mark:selected', {
      detail: {
        discussionId: this.props.node.target,
        markPosition: position
      }
    }))
  }

}

export default MarkComponent
