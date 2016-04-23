'use strict';

module.exports = {

  type: 'stencil-codeblock',
  tagName: 'pre',

  matchElement: function(el) {
    if (el.is('pre') && el.children.length==1) {
      return el.children[0].attr('data-code') !== undefined;
    }
    return false;
  },

  import: function(el, node) {
    var code = el.find('[data-code]');
    node.lang = code.attr('data-code');
    node.source = code.text().trim();
  },

  export: function(node, el) {
    el
    .attr('data-code', node.lang)
    .text(node.source);
  }
};
