'use strict';

var Component = require('substance/ui/Component');
var TextPropertyEditor = require('substance/ui/TextPropertyEditor');

function ImageMarkdownComponent () {
  ImageMarkdownComponent.super.apply(this, arguments);
}

ImageMarkdownComponent.Prototype = function () {
  this.render = function ($$) {
    var node = this.props.node;
    return $$('span')
      .addClass('sc-image')
      .append(
        '![](',
        $$(TextPropertyEditor, {
          path: [ node.id, 'src' ],
          withoutBreak: true
        }),
        ')'
      );
  };
};

Component.extend(ImageMarkdownComponent);

module.exports = ImageMarkdownComponent;
