'use strict';

import IsolatedNodeComponent from 'substance/ui/IsolatedNodeComponent'
import ContainerEditor from 'substance/ui/ContainerEditor'

import moment from 'moment'

function CommentComponent () {
  CommentComponent.super.apply(this, arguments);

  this.ContentClass = ContainerEditor;
}

CommentComponent.Prototype = function () {
  var _super = CommentComponent.super.prototype;

  /**
   * Method override for custom class names
   */
  this.getClassNames = function () {
    return 'sc-comment';
  };

  /**
   * Method override to disable the comment unless the current user
   * is the original author of the comment
   */
  this.isDisabled = function () {
    var user = this.context.documentSession.config.user;
    return this.props.node.who !== ('@' + user);
  };

  /**
   * Method override so no blocker is rendered over this
   * `IsolatedNodeComponent` (requires two clicks to begin editing)
   */
  this.shouldRenderBlocker = function () {
    // CHECK Is this method needed?
    return false;
  };

  /**
   * Method ovveride to add additional elements
   */
  this.render = function ($$) {
    var node = this.props.node;
    return _super.render.call(this, $$)
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
      );
  };
};

IsolatedNodeComponent.extend(CommentComponent);

module.exports = CommentComponent;
