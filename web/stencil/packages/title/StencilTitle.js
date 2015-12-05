'use strict';

var TextBlock = require('substance/model/TextBlock');

function StencilTitle() {
  StencilTitle.super.apply(this, arguments);
}

TextBlock.extend(StencilTitle);

StencilTitle.static.name = "stencil-title";

StencilTitle.static.defineSchema({
  'content': 'text'
});

module.exports = StencilTitle;
