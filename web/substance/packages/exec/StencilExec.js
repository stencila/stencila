
var $ = require('substance/util/jquery');
var StencilNode = require('../../model/StencilNode');

// Abstract interface
// There are ImageFigures, TableFigures, VideoFigures

// <pre id="exec1", data-exec="r">
// x = 1
// </pre>

var StencilExec = StencilNode.extend({
  name: "stencil-exec",
  properties: {
    "source": "string",
    "error": "string",
    "spec": "string"
  }
});

// declare editable components, so that we can enable ContainerEditor features
StencilExec.static.components = ['spec'];

StencilExec.static.generatedProps = ['error'];

StencilExec.static.blockType = true;

StencilExec.static.matchElement = function($el) {
  return $el.is('pre') && $el.attr('data-exec');
};

// HtmlImporter

StencilExec.static.fromHtml = function($el, converter) {
  var id = converter.defaultId($el, 'stencil-exec');
  var source = $el.text();
  var error = $el.attr('data-error');
  var spec = $el.attr('data-exec');

  var exec = {
    id: id,
    source: source,
    error: error,
    spec: spec
  };

  return exec;
};

// HtmlExporter

StencilExec.static.toHtml = function(exec) {
  var id = exec.id;
  var $el = $('<pre>')
    .attr('id', id)
    .attr('data-exec', exec.spec)
    .attr('data-error', exec.error)
    .text(exec.source);

  return $el;
};

module.exports = StencilExec;
