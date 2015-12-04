'use strict';

var TextNode = require('substance/model/TextNode');

function StencilTitle() {
  StencilTitle.super.call(this, arguments);
}

TextNode.extend(StencilTitle);

StencilTitle.static.name = "stencil-title";

StencilTitle.static.defineSchema({
  'content': 'text'
});

module.exports = StencilTitle;
