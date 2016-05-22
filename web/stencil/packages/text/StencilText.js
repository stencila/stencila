'use strict';

var InlineNode = require('substance/model/InlineNode');
var StencilNode = require('../../model/StencilNode');

function StencilText(){
  StencilText.super.apply(this, arguments);
}

InlineNode.extend(StencilText, StencilNode);

StencilText.static.name = 'stencil-text';

StencilText.static.defineSchema({
  'tagName': { type: 'string', optional: true },
  'source':  { type: 'string', optional: true },
  'error':   { type: 'string', optional: true },
  'output':  { type: 'string', optional: true }
});

StencilText.static.generatedProps = ['error', 'output'];

module.exports = StencilText;
