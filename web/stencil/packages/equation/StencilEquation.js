"use strict";

var StencilNode = require('../../model/StencilNode');

function StencilEquation(){
  StencilEquation.super.call(this, arguments);
}

StencilNode.extend(StencilEquation);

StencilEquation.static.name = "stencil-exec";

StencilEquation.static.defineSchema({
  source: { type: "string", default: "" },
  error: { type: "string", optional: true },
  format: "string"
});

StencilEquation.static.generatedProps = ['error'];

StencilEquation.static.isBlock = true;

module.exports = StencilEquation;
