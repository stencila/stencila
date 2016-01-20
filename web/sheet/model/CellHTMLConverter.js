'use strict';

var Sheet = require('./Sheet');

module.exports = {

  type: 'sheet-cell',
  tagName: 'td',

  matchElement: function(el) {
    return el.is('td');
  },

  import: function(el, node) {
    var textContent = el.textContent;
    var expr = el.attr('data-expr');
    var name = el.attr('data-name');
    var valueType = el.attr('data-type');
    if (Sheet.isPrimitiveType(valueType)) {
      node.content = textContent;
    } else if (name) {
      node.content = node.name + '=' + node.expr;
    } else {
      node.content = expr;
    }
    node.valueType = valueType;
    node.value = textContent;
    if (name) {
    } else if (expr) {
    } else {
      node.content = el.text();
    }
    if (valueType) {
      node.valueType = valueType;
    }
    node.value = textContent;
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
