"use strict";

var DocumentNode = require('substance/model/DocumentNode');
var StencilNode = require('../../model/StencilNode');

function StencilEquation(){
  StencilEquation.super.call(this, arguments);
}

DocumentNode.extend(StencilEquation, StencilNode);

StencilEquation.static.name = "stencil-equation";

StencilEquation.static.defineSchema({
  source: { type: "string", default: "" },
  format: "string"
});

StencilEquation.static.generatedProps = ['error'];

StencilEquation.static.isBlock = true;

module.exports = StencilEquation;
