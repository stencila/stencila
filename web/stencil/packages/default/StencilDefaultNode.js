"use strict";

var BlockNode = require('substance/model/BlockNode');

// Abstract interface
// There are ImageFigures, TableFigures, VideoFigures

function StencilDefaultNode() {
  StencilDefaultNode.super.apply(this, arguments);
}

BlockNode.extend(StencilDefaultNode);

StencilDefaultNode.static.name = "stencil-default-node";

StencilDefaultNode.static.defineSchema({
  "html": "string"
});

module.exports = StencilDefaultNode;