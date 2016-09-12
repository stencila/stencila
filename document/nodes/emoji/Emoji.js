'use strict';

var InlineNode = require('substance/model/InlineNode');

function Emoji () {
  Emoji.super.apply(this, arguments);
}

InlineNode.extend(Emoji);

Emoji.define({
  type: 'emoji',
  name: { type: 'string' }
});

module.exports = Emoji;
