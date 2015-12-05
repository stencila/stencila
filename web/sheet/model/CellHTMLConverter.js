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
    var alias = el.attr('data-alias');
    if (alias) {
      node.alias = alias;
    }
  },

  export: function(node, el) {
    if (node.isExpression()) {
      el.attr('data-expr', node.getExpression());
    }
    el.text(node.getValue());
    if (node.alias) {
      el.attr('data-alias', node.alias);
    }
  }
};
