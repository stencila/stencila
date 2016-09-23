'use strict'

import AnnotationComponent from 'substance/ui/AnnotationComponent'
import getRelativeBoundingRect from 'substance/util/getRelativeBoundingRect'

function MarkComponent () {
  MarkComponent.super.apply(this, arguments)
}

MarkComponent.Prototype = function () {
  var _super = MarkComponent.super.prototype

  this.render = function ($$) {
    var el = _super.render.call(this, $$)
    el.on('click', this._selected, this)
    return el
  }

  /**
   * When a mark is selected notify the associated `DiscussionComponent`
   * to show itself
   */
  this._selected = function () {
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

AnnotationComponent.extend(MarkComponent)

export default MarkComponent
