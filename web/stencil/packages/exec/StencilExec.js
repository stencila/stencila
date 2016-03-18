'use strict';

var DocumentNode = require('substance/model/DocumentNode');
var StencilNode = require('../../model/StencilNode');

function StencilExec() {
  StencilExec.super.apply(this, arguments);
}

DocumentNode.extend(StencilExec, StencilNode);

StencilExec.static.name = "stencil-exec";

StencilExec.static.defineSchema({
  source: { type: "string", default: "" },
  spec: "string",
  lang: { type: "string", optional: true },
  show: { type: "boolean", default: false },
  error: { type: "string", optional: true }
});

StencilExec.static.generatedProps = ['error'];

StencilExec.static.isBlock = true;

module.exports = StencilExec;

