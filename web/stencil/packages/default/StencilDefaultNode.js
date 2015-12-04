"use strict";

var $ = require('substance/util/jquery');
var BlockNode = require('substance/model/BlockNode');
var helpers = require('substance/util/helpers');

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

