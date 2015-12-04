"use strict";

var StencilNode = require('../../model/StencilNode');

function StencilFigure() {
  StencilFigure.super.call(this, arguments);
}

StencilNode.extend(StencilFigure);

StencilFigure.static.name = "stencil-figure";

StencilFigure.static.defineSchema({
    'index': 'string',
    'spec': 'string',
    'source': 'string',
    'error': 'string',
    'image': 'string',
    'label': 'string',
    'caption': 'string'
});

StencilFigure.static.generatedProps = [
  'image', 'error', 'label', 'index'
];

StencilFigure.static.isBlock = true;

StencilFigure.static.components = ['caption'];

module.exports = StencilFigure;
