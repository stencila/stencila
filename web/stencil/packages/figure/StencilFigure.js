
var $ = require('substance/util/jquery');
var StencilNode = require('../../model/StencilNode');

var StencilFigure = StencilNode.extend({
  name: 'stencil-figure',
  properties: {
    'index': 'string',
    'spec': 'string',
    'source': 'string',
    'error': 'string',
    'image': 'string',
    'label': 'string',
    'caption': 'string'
  }
});

StencilFigure.static.generatedProps = [
  'image', 'error', 'label', 'index'
];

StencilFigure.static.components = ['caption'];

StencilFigure.static.blockType = true;

StencilFigure.static.matchElement = function($el) {
  return $el.is('figure');
};

StencilFigure.static.fromHtml = function($el, converter) {
  var id = converter.defaultId($el, 'stencil-figure');

  var figure = {
    id: id,
    index: '',
    spec: null,
    source: null,
    hash: null,
    error: null,
    image: null,
    caption: ''
  };

  figure.index = $el.attr('data-index');

  var $exec = $el.find('[data-exec]');
  if($exec.length){
    figure.spec = $exec.attr('data-exec');
    figure.source = $exec.text();
    figure.hash = $exec.attr('data-hash');
    figure.error = $exec.attr('data-error');
  }

  var $img = $el.find('[data-out] img');
  if($img.length){
    figure.image = $img.attr('src');
  }

  var $caption = $el.find('figcaption,caption');
  if($caption.length){
    figure.caption = converter.annotatedText($caption, [id, 'caption']);
  }

  return figure;
};

StencilFigure.static.toHtml = function(figure, converter) {
  var id = figure.id;

  var $el = $('<figure>')
    .attr('id', id);
  if(figure.index) $el.attr('data-index',figure.index);

  var $exec = $('<pre>')
    .attr('data-exec',figure.spec)
    .text(figure.source);
  if(figure.hash) $exec.attr('data-hash',figure.hash);
  if(figure.error) $exec.attr('data-error',figure.error);

  var $img = $('<img>')
    .attr('src',figure.image);

  var $out = $('<div>')
    .attr('data-out','true')
    .append($img);

  var $caption = $('<figcaption>')
    .append(converter.annotatedText([id, 'caption']));

  return $el.append($exec, $out, $caption);
};

module.exports = StencilFigure;
