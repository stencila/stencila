var $ = require('substance/util/jquery');

var Annotation = require('substance/model/Annotation');
var StencilNode = require('../../model/StencilNode');

var StencilMath = Annotation.extend({
  name: 'stencil-math',
  properties: {
    'source': 'string',
    'format': 'string',
    'error': 'string'
  }
}, StencilNode.prototype);

StencilMath.static.components = [];
StencilMath.static.external = true;
StencilMath.static.generatedProps = [
  'error'
];

StencilMath.static.tagName = ['span'];

StencilMath.static.matchElement = function($el) {
  return $el.is('script[type^="math/tex"],script[type^="math/asciimath"]');
};

StencilMath.static.fromHtml = function($el, converter) {
  var id = converter.defaultId($el, 'stencil-math');
  var math = {
    id: id,
    source: null,
    error: null
  };
  var format = $el.prop('type');
  format = format.split(';')[0];
  math.source = $el.text();
  math.format = format;
  return math;
};

StencilMath.static.toHtml = function(math, converter) {
  return $('<script>')
    .attr('id', math.id)
    .prop('type', math.format)
    .text(math.source);
};

module.exports = StencilMath;
