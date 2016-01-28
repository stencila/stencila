"use strict";

var InlineNode = require('substance/model/InlineNode');
var StencilNode = require('../../model/StencilNode');

function StencilMath() {
  StencilMath.super.apply(this, arguments);
}

InlineNode.extend(StencilMath, StencilNode);

StencilMath.static.name = "stencil-math";

StencilMath.static.defineSchema({
  'source': 'string',
  'format': 'string',
  'error': { type:'string', optional: true }
});

StencilMath.static.generatedProps = ['error'];

module.exports = StencilMath;
