'use strict';

module.exports = {

  type: 'stencil-exec',
  tagName: 'pre',

  matchElement: function(el) {
    return el.is('pre') && el.attr('data-exec');
  },

  import: function(el, node, converter) {
    node.source = el.text();
    node.error = el.attr('data-error');
    node.spec = el.attr('data-exec');
  },

  export: function(node, el, converter) {
    el
    .attr('data-exec', node.spec)
    .attr('data-error', node.error)
    .text(node.source);
  }
};
