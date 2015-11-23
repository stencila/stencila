'use strict';

var $ = require('substance/util/jquery');
var TextNode = require('substance/model/TextNode');

var StencilTitle = TextNode.extend({
  name: 'stencil-title',
  displayName: 'Title',
  properties: {
    'content': 'string'
  }
});

StencilTitle.static.blockType = true;

StencilTitle.static.matchElement = function($el) {
  return $el.is('div#title');
};

StencilTitle.static.fromHtml = function($el, converter) {
  return {
    id: converter.defaultId($el, 'stencil-title'),
    content: $el.text()
  };
};

StencilTitle.static.toHtml = function(title, converter) {
  return $('<div>')
    .attr('id', title.id)
    .text(title.content);
};

module.exports = StencilTitle;
