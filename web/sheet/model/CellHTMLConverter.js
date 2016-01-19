'use strict';

module.exports = {

  type: 'sheet-cell',
  tagName: 'td',

  matchElement: function(el) {
    return el.is('td');
  },

  import: function(el, node) {
    var contentType = el.attr('data-type');
    if (contentType) {
      node.contentType = contentType;
    }
    var expr = el.attr('data-expr');
    var name = el.attr('data-name');
    if (name) {
      node.source = node.name + '=' + node.expr;
    } else {
      node.source = node.expr;
    }

    node.value = el.text();
  },

  export: function(node, el) {
    if (node.contentType) {
      el.attr('data-type', node.contentType);
    }

    var name = node.getName();
    if (name) {
      el.attr('data-name', name);
    }
    el.attr('data-expr', node.source);

    // using getValue() here as it is evaluated dynamically
    el.text(node.getValue());
  }
};
