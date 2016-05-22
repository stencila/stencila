'use strict';

var DocumentNode = require('substance/model/DocumentNode');
var StencilNode = require('../../model/StencilNode');

function StencilInclude() {
  StencilInclude.super.apply(this, arguments);
}
DocumentNode.extend(StencilInclude, StencilNode);

StencilInclude.static.name = "stencil-include";
StencilInclude.static.directive = 'include';
StencilInclude.static.defineSchema({
  source:  { type: "string", default: "" },
  error:   { type: "string", default: "" },
  content: { type: "string", default: "" }
});
StencilInclude.static.generatedProps = ['error', 'content'];
StencilInclude.static.isBlock = true;

module.exports = StencilInclude;

