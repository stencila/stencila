'use strict';

var IsolatedNodeComponent = require('substance/ui/IsolatedNodeComponent');

var CommentComponent = require('./CommentComponent');

function CommentMarkdownComponent () {

  CommentMarkdownComponent.super.apply(this, arguments);

}

CommentMarkdownComponent.Prototype = function () {

  this.render = function ($$) {

    var node = this.props.node;
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
      );

  };

};

CommentComponent.extend(CommentMarkdownComponent);

module.exports = CommentMarkdownComponent;
