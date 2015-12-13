'use strict';

module.exports = {

  type: 'sheet-cell',
  tagName: 'td',

  matchElement: function(el) {
    return el.is('td');
  },

  import: function(el, node) {
    var expr = el.attr('data-expr');
    if (expr) {
      node.expr = expr;
    }
    var name = el.attr('data-name');
    if (name) {
      node.name = name;
    }
    node.value = el.text();
  },

  export: function(node, el) {
    if (node.expr) {
      el.attr('data-expr', node.expr);
    }
    if (node.name) {
      el.attr('data-name', node.name);
    }
    el.text(node.value);
  }
};
