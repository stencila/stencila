'use strict';

var DocumentNode = require('substance/model/DocumentNode');
var StencilNode = require('../../model/StencilNode');

function StencilFigure() {
  StencilFigure.super.apply(this, arguments);
}

DocumentNode.extend(StencilFigure, StencilNode);

StencilFigure.static.name = "stencil-figure";

StencilFigure.static.defineSchema({
    'spec': 'string',
    'source': 'string',
    'caption': {type: 'text', default: 'Caption'},

    // Properties rendered by the backend
    // 
    // Index (ie. number) of the figure
    'index': { type: 'string', optional: true },
    // Rendered image 
    'image': { type: 'string', default: "http://" },
    // Image styling
    // Currently, the size specified in the exec spec is
    // applied to the image's style attribute
    // In the future it could be parsed from the spec itself
    // but this is currently the easiest way to do it.
    'image_style': { type: 'string', optional: true },
    // Any error associated with execution
    'error': { type: 'string', optional: true },
    // Extra meta-data associated with the element
    'extra': { type: 'string', optional: true },
});

StencilFigure.static.generatedProps = [
  'index', 'image', 'image_style', 'error'
];

StencilFigure.static.isBlock = true;

module.exports = StencilFigure;
