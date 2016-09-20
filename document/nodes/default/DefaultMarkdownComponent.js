'use strict';

import Component from 'substance/ui/Component'

var CodeEditorComponent = require('../../ui/CodeEditorComponent');

function DefaultMarkdownComponent () {
  DefaultMarkdownComponent.super.apply(this, arguments);
}

DefaultMarkdownComponent.Prototype = function () {
  var _super = DefaultMarkdownComponent.super.prototype;

  this.render = function ($$) {
    var node = this.props.node;
    return _super.render.call(this, $$)
      .addClass('sc-default')
      .append(
        $$(CodeEditorComponent, {
          node: node,
          codeProperty: 'html',
          languageProperty: null,
          language: 'html'
        }).ref('code')
      );
  };
};

Component.extend(DefaultMarkdownComponent);

DefaultMarkdownComponent.fullWidth = true;

module.exports = DefaultMarkdownComponent;
