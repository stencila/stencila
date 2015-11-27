'use strict';

var $ = require('substance/util/jquery');
var TextNode = require('substance/model/TextNode');

var StencilSummary = TextNode.extend({
  name: 'stencil-summary',
  displayName: 'Summary',
  properties: {
    'content': 'string'
  }
});

StencilSummary.static.blockType = true;

StencilSummary.static.matchElement = function($el) {
  return $el.is('div#summary, div#description');
};

StencilSummary.static.fromHtml = function($el, converter) {
  return {
    id: converter.defaultId($el, 'stencil-summary'),
    content: $el.text()
  };
};

StencilSummary.static.toHtml = function(summary, converter) {
  return $('<div>')
    .attr('id', summary.id)
    .text(summary.content);
};

module.exports = StencilSummary;
