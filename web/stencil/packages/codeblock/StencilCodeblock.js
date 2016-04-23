'use strict';

var DocumentNode = require('substance/model/DocumentNode');
var StencilNode = require('../../model/StencilNode');

function StencilCodeblock() {
  StencilCodeblock.super.apply(this, arguments);
}
DocumentNode.extend(StencilCodeblock, StencilNode);

StencilCodeblock.static.name = "stencil-codeblock";

StencilCodeblock.static.defineSchema({
  source: { type: "string", default: "" },
  lang:   { type: "string", default: "" },
});

StencilCodeblock.static.isBlock = true;

module.exports = StencilCodeblock;

