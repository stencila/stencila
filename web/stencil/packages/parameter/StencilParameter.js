'use strict';

var DocumentNode = require('substance/model/DocumentNode');
var StencilNode = require('../../model/StencilNode');

function StencilParameter() {
  StencilParameter.super.apply(this, arguments);
}

DocumentNode.extend(StencilParameter, StencilNode);

StencilParameter.static.name = "stencil-parameter";

StencilParameter.static.defineSchema({
    'name': 'text',
    'tipe': { type: 'string', optional: true },
    'default': { type: 'string', optional: true },
    'value': { type: 'text', optional: true },
    'error': { type: 'string', optional: true },
});

StencilParameter.static.generatedProps = ['error'];

StencilParameter.static.isBlock = true;

module.exports = StencilParameter;
