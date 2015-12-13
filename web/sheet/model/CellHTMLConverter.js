'use strict';

module.exports = {

  type: 'sheet-cell',
  tagName: 'td',

  matchElement: function(el) {
    return el.is('td');
  },

  import: function(el, node) {
    // the rendered content
    node.value = el.text();
    var expr = el.attr('data-expr');
    if (expr) {
      node.content = "=" + expr;
    } else {
      // in case of a text content node.value === node.content
      node.content = node.value;
    }
    var name = el.attr('data-name');
    if (name) {
      node.name = name;
    }
  },

  export: function(node, el) {
    if (node.isExpression()) {
      el.attr('data-expr', node.getExpression());
    }
    el.text(node.getValue());
    if (node.name) {
      el.attr('data-name', node.name);
    }
  }
};
