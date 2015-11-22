var Annotation = require('substance/model/Annotation');
var StencilNode = require('../../model/StencilNode');
var StencilEquation = require('./StencilEquation');

var StencilFormula = Annotation.extend({
  name: 'stencil-formula',
  properties: {
    'source': 'string',
    'format': 'string',
    'error': 'string'
  }
}, StencilNode.prototype);

StencilFormula.static.components = [];
StencilFormula.static.external = true;
StencilFormula.static.generatedProps = ['error'];

StencilFormula.static.tagName = ['span'];

StencilFormula.static.matchElement = function($el) {
  return $el.is('script[type^="math/tex"],script[type^="math/asciimath"]');
};

StencilFormula.static.fromHtml = function($el, converter) {
  var id = converter.defaultId($el, 'stencil-formula');
  var formula = {
    id: id,
    source: null,
    error: null
  };
  var format = $el.prop('type');
  format = format.split(';')[0];
  formula.source = $el.text();
  formula.format = format;
  return formula;
};

StencilFormula.static.toHtml = function(formula, converter) {
  var id = equation.id;
  var $el = $('<script>').attr('id', id)
    .prop('type', equation.format)
    .text(equation.source);
  return $el;
};

module.exports = StencilFormula;
