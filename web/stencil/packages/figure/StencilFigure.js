"use strict";

var DocumentNode = require('substance/model/DocumentNode');
var StencilNode = require('../../model/StencilNode');

function StencilFigure() {
  StencilFigure.super.call(this, arguments);
}

DocumentNode.extend(StencilFigure, StencilNode);

StencilFigure.static.name = "stencil-figure";

StencilFigure.static.defineSchema({
    'index': 'string',
    'spec': 'string',
    'source': 'string',
    'error': 'string',
    'image': 'string',
    'label': 'string',
    'caption': 'text'
});

StencilFigure.static.generatedProps = [
  'image', 'error', 'label', 'index'
];

StencilFigure.static.isBlock = true;

module.exports = StencilFigure;
