'use strict';

var Component = require('substance/ui/Component');
var TextPropertyComponent = require('substance/ui/TextPropertyComponent');

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
        $$(TextPropertyComponent, {
          path: [ node.id, 'src' ],
          withoutBreak: true
        }),
        ')'
      );
  };
};

Component.extend(ImageMarkdownComponent);

module.exports = ImageMarkdownComponent;
