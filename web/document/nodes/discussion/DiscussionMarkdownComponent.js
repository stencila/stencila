'use strict';

var IsolatedNodeComponent = require('substance/ui/IsolatedNodeComponent');
var ContainerEditor = require('substance/ui/ContainerEditor');

function DiscussionMarkdownComponent () {

  DiscussionMarkdownComponent.super.apply(this, arguments);

  this.ContentClass = ContainerEditor;

}

DiscussionMarkdownComponent.Prototype = function () {

  var _super = DiscussionMarkdownComponent.super.prototype;

  this.render = function ($$) {

    return _super.render.call(this, $$)
      .addClass('sc-discussion');

  };

};

IsolatedNodeComponent.extend(DiscussionMarkdownComponent);

module.exports = DiscussionMarkdownComponent;
