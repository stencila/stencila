'use strict';

module.exports = {

  type: 'stencil-default-node',

  import: function(el, node) {
    node.html = el.outerHTML;
  },

  export: function(node, el, converter) {
    // instead of using the prepopulated one
    // we create a new one from the HTML stored in the node
    el = converter.$$(node.html);
    el.attr('data-id', node.id);
    return el;
  },

};
