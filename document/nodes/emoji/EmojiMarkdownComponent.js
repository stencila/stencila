'use strict';

import EmojiComponent from './EmojiComponent'

function EmojiMarkdownComponent () {
  EmojiMarkdownComponent.super.apply(this, arguments);
}

EmojiMarkdownComponent.Prototype = function () {
  this.render = function ($$) {
    var node = this.props.node;
    return $$('span')
      .addClass('sc-emoji')
      .text(':' + node.name + ':');
  };
};

EmojiComponent.extend(EmojiMarkdownComponent);

module.exports = EmojiMarkdownComponent;
