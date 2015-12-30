'use strict';

var InlineNode = require('substance/model/InlineNode');
var StencilNode = require('../../model/StencilNode');

function StencilText(){
  StencilText.super.apply(this, arguments);
}

InlineNode.extend(StencilText, StencilNode);

StencilText.static.name = "stencil-text";

StencilText.static.defineSchema({
  'tagName': 'string',
  'source': 'string',
  'error': { type: 'string', optional: true },
  'output': 'string'
});

StencilText.static.generatedProps = ['error', 'output'];

module.exports = StencilText;
