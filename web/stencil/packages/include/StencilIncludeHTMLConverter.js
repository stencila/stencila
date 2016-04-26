'use strict';

module.exports = {

  type: 'stencil-include',
  tagName: 'div',

  matchElement: function(el) {
    return el.attr('data-include');
  },

  import: function(el, node) {
    node.source = el.attr('data-include');
    node.content = el.html();
  },

  export: function(node, el) {
    el.attr('data-include', node.source);
    el.html(node.content);
  }
};
