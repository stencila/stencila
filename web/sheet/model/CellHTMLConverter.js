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
    var displayMode = el.attr('data-display-mode');
    var exprType = el.attr('data-kind');
    var valueType = el.attr('data-type');
    // FIXME: we should agree on a set of valueTypes
    valueType = Sheet.normalizeValueType(valueType);
    // primitives
    if (exprType === 's' || exprType === 'z' || exprType === 'n') {
      node.content = textContent;
    }
    // expressions
    else {
      if (name) {
        node.content = name + '=' + expr;
      } else {
        node.content = '=' + expr;
      }
    }
    node.value = textContent;
    node.valueType = valueType;
    node.displayMode = displayMode;
  },

  export: function(node, el) {
    var contentType = node.getContentType();
    switch(contentType) {
      case 'primitive':
        el.text(node.content);
        break;
      case 'expression':
        el.attr('data-expr', node.getExpression());
        break;
      case 'named-expression':
        el.attr('data-name', node.getName());
        el.attr('data-expr', node.getExpression());
        break;
      default:
        throw new Error('Illegal content type.', contentType);
    }
    el.attr('data-display-mode', node.displayMode);
  }
};
