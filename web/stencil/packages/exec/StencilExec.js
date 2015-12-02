"use strict";

var StencilNode = require('../../model/StencilNode');

function StencilExec() {
  StencilExec.super.call(this, arguments);
}

StencilNode.extend(StencilExec);

StencilExec.static.name = "stencil-exec";

StencilExec.static.defineSchema({
  source: { type: "string", default: "" },
  error: { type: "string", optional: true },
  spec: "string"
});

StencilExec.static.generatedProps = ['error'];

StencilExec.static.isBlock = true;

module.exports = StencilExec;

