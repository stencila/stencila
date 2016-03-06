'use strict';

var Cell = require('./Cell');

module.exports = {

  type: 'sheet-cell',
  tagName: 'td',

  matchElement: function(el) {
    return el.is('td');
  },

  import: function(el, node) {
    var name = el.attr('data-name');
    var kind = el.attr('data-kind');
    var expr = el.attr('data-expr');
    var displayMode = el.attr('data-display');
    var valueType = el.attr('data-type');
    var value = el.textContent;
    
    var symbol = Cell.static.kindToSymbol(kind);
    if (symbol) {
      if (name) {
        node.content = name + ' ' + symbol + ' ' + expr;
      } else {
        node.content = symbol + ' ' + expr;
      }
    } else {
      node.content = value;
    }

    node.kind = kind;
    node.value = value;
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
