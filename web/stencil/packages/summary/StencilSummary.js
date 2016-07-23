'use strict';

var TextNode = require('substance-fe0ed/model/TextNode');

function StencilSummary() {
  StencilSummary.super.apply(this, arguments);
}

TextNode.extend(StencilSummary);

StencilSummary.static.name = "stencil-summary";

StencilSummary.static.defineSchema({
  'content': 'text'
});

module.exports = StencilSummary;
