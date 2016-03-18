"use strict";

var BlockNode = require('substance/model/BlockNode');

function StencilDefaultNode() {
  StencilDefaultNode.super.apply(this, arguments);
}

BlockNode.extend(StencilDefaultNode);

StencilDefaultNode.static.name = "stencil-default-node";

StencilDefaultNode.static.defineSchema({
  "html": { type: 'string', optional: true }
});

module.exports = StencilDefaultNode;