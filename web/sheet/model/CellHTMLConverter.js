'use strict';

module.exports = {

  type: 'sheet-cell',
  tagName: 'td',

  matchElement: function(el) {
    return el.is('td');
  },

  import: function(el, node) {
    var expr = el.attr('data-expr');
    var name = el.attr('data-name');
    if (name) {
      node.content = node.name + '=' + node.expr;
    } else {
      node.content = node.expr;
    }
    var valueType = el.attr('data-type');
    if (valueType) {
      node.valueType = valueType;
    }
    node.value = el.text();
  },

  export: function(node, el) {
    var name = node.getName();
    if (name) {
      el.attr('data-name', name);
    }
    el.attr('data-expr', node.content);
    if (node.valueType) {
      el.attr('data-type', node.valueType);
    }
    // using getValue() here as it is evaluated dynamically
    el.text(node.getValue());
  }
};
