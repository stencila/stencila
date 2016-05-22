'use strict';

var DocumentNode = require('substance/model/DocumentNode');
var StencilNode = require('../../model/StencilNode');

function StencilOut() {
  StencilOut.super.apply(this, arguments);
}
DocumentNode.extend(StencilOut, StencilNode);

StencilOut.static.name = "stencil-out";

StencilOut.static.defineSchema({
    'content': 'string'
});

StencilOut.static.generatedProps = [
  'content'
];

StencilOut.static.isBlock = true;

module.exports = StencilOut;
