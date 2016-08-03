'use strict';

var IsolatedNodeComponent = require('substance/ui/IsolatedNodeComponent');
var ContainerEditor = require('substance/ui/ContainerEditor');

var moment = require('moment');


function CommentComponent() {
  CommentComponent.super.apply(this, arguments);

  this.ContentClass = ContainerEditor;
}

CommentComponent.Prototype = function() {

  var _super = CommentComponent.super.prototype;

  /**
   * Method override for custom class names
   */
  this.getClassNames = function() {
    return 'sc-comment';
  }

  /**
   * Method override so no blocker is rendered over this
   * `IsolatedNodeComponent` (requires two clicks to begin editing)
   */
  this.shouldRenderBlocker = function() {
    return false;
  }

  /**
   * Method ovveride to add additional elements
   */
  this.render = function($$) {
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
  }

};

IsolatedNodeComponent.extend(CommentComponent);


module.exports = CommentComponent;
