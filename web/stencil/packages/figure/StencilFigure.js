"use strict";

var DocumentNode = require('substance/model/DocumentNode');
var StencilNode = require('../../model/StencilNode');

function StencilFigure() {
  StencilFigure.super.apply(this, arguments);
}

DocumentNode.extend(StencilFigure, StencilNode);

StencilFigure.static.name = "stencil-figure";

StencilFigure.static.defineSchema({
    'index': { type: 'string', optional: true },
    'spec': 'string',
    'source': 'string',
    'error': { type: 'string', optional: true },
    'image': { type: 'string', default: "http://" },
    'label': { type: 'string', optional: true },
    'caption': 'text'
});

StencilFigure.static.generatedProps = [
  'image', 'error', 'label', 'index'
];

StencilFigure.static.isBlock = true;

module.exports = StencilFigure;
