'use strict';

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
    var displayMode = el.attr('data-display-mode');
    var valueType = el.attr('data-type');
    var value = el.textContent;
    
    // This is repetative of what is in Cell.getPrefix()
    // and it might be better just to have a Cell.getSource()
    // function instead of generating here and then again elsewhere
    var symbol = '';
    if (kind=='exp') symbol = '=';
    else if (kind=='map') symbol = '~';
    else if (kind=='req') symbol = '^';
    else if (kind=='man') symbol = ':';
    else if (kind=='tes') symbol = '?';
    else if (kind=='vis') symbol = '|';
    else {
      kind = 'lit';
      symbol = '';
    }
    node.kind = kind;

    if (symbol) {
      if (name) {
        node.content = name + symbol + expr;
      } else {
        node.content = symbol + expr;
      }
    } else {
      node.content = value;
    }

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
