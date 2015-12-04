"use strict";

var InlineNode = require('substance/model/InlineNode');
var StencilNode = require('../../model/StencilNode');

function StencilMath() {
  StencilMath.super.call(this, arguments);
}

InlineNode.extend(StencilMath, StencilNode);

StencilMath.static.name = "stencil-math";

StencilMath.static.defineSchema({
  'source': 'string',
  'format': 'string',
  'error': 'string'
});

StencilMath.static.generatedProps = ['error'];

module.exports = StencilMath;
