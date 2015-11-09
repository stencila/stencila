
var $ = require('substance/util/jquery');
var DocumentNode = require('substance/model/DocumentNode');
var helpers = require('substance/util/helpers');

// Abstract interface
// There are ImageFigures, TableFigures, VideoFigures

var StencilDefaultNode = DocumentNode.extend({
  name: "stencil-default-node",
  properties: {
    // "source": "string",
    "html": "string"
  }
});

// declare editable components, so that we can enable ContainerEditor features
StencilDefaultNode.static.components = [];
StencilDefaultNode.static.blockType = true;

// HtmlImporter

StencilDefaultNode.static.fromHtml = function($el, converter) {
  var id = converter.defaultId($el, 'stencil-default-node');
  var html = helpers.serializeDOMElement($el);

  var defaultNode = {
    id: id,
    html: html
  };

  return defaultNode;
};

// HtmlExporter

StencilDefaultNode.static.toHtml = function(defaultNode) {
  return $(defaultNode.html);
};

module.exports = StencilDefaultNode;
