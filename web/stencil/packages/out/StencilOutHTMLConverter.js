'use strict';

module.exports = {

  type: 'stencil-out',
  tagName: 'div',

  matchElement: function(el) {
    return el.is('[data-out]');
  },

  import: function(el, node, converter) {
    node.content = el.innerHTML;
  },

  export: function(node, el, converter) {
    el.attr("data-out", "true")
      .html(node.content);
  }
};
