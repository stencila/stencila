var $ = require('substance/util/jquery');

var StencilNode = require('../../model/StencilNode');

var StencilEquation = StencilNode.extend({
  name: 'stencil-equation',
  properties: {
    'source': 'string',
    'format': 'string',
    'error': 'string',
  }
});

StencilEquation.static.generatedProps = [
  'error'
];

StencilEquation.static.components = [];

StencilEquation.static.blockType = true;

StencilEquation.static.matchElement = function($el) {
  return $el.is('div[data-equation]');
};

StencilEquation.static.fromHtml = function($el, converter) {
  var id = converter.defaultId($el, 'stencil-equation');
  var equation = {
    id: id,
    source: null,
    error: null
  };
  var $source = $el.find('script');
  if($source.length > 0){
    var format = $source.prop('type');
    format = format.split(';')[0];
    equation.source = $source.text();
    equation.format = format;
  }
  return equation;
};

StencilEquation.static.toHtml = function(equation, converter) {
  var id = equation.id;
  var $el = $('<div>')
    .attr('id', id)
    .attr('data-equation', 'true');
  var $script = $('<script>')
    .prop('type', equation.format)
    .text(equation.source);
  $el.append($script);
  return $el;
};

module.exports = StencilEquation;
