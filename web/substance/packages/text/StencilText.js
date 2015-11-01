var $ = require('substance/util/jquery');
var Annotation = require('substance/model/Annotation');

var StencilText = Annotation.extend({
  name: 'stencil-text',
  properties: {
    'tag': 'string',
    'source': 'string',
    'error': 'string',
    'output': 'string'
  }
});

StencilText.static.components = [];
StencilText.static.external = true;

StencilText.static.matchElement = function($el) {
  return $el.attr('data-text');
};

StencilText.static.fromHtml = function($el, converter) {
  var source = $el.attr('data-text');
  var node = {
    id: converter.defaultId($el, 'stencil-text'),
    tag: $el[0].tagName.toLowerCase(),
    source: source,
    error: $el.attr('data-error'),
    output: $el.text()
  };

  return node;
};

StencilText.static.toHtml = function(text) {
  var $el = $('<'+text.tag+'>')
    .attr('id', text.id)
    .attr('data-text',text.source);
  if(text.error) $el.attr('data-error',text.error);
  $el.text(text.output);
  return $el;
};

module.exports = StencilText;
