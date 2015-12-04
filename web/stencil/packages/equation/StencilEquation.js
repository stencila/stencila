"use strict";

var StencilNode = require('../../model/StencilNode');

function StencilEquation(){
  StencilEquation.super.call(this, arguments);
}

StencilNode.extend(StencilEquation);

StencilEquation.static.name = "stencil-equation";

StencilEquation.static.defineSchema({
  source: { type: "string", default: "" },
  format: "string"
});

StencilEquation.static.generatedProps = ['error'];

StencilEquation.static.isBlock = true;

module.exports = StencilEquation;
