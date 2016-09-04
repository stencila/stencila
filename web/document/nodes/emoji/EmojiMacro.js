'use strict';

var InlineNodeMacro = require('../../ui/InlineNodeMacro');

function EmojiMacro () {
};

EmojiMacro.Prototype = function () {
  this.regex = /:([a-z0-9_]+):/;

  this.createNodeData = function (match) {
    var name = match[1];
    return {
      type: 'emoji',
      name: name
    };
  };
};

InlineNodeMacro.extend(EmojiMacro);

module.exports = EmojiMacro;
