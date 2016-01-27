'use strict';

module.exports = {

  type: 'stencil-parameter',
  tagName: 'div',

  matchElement: function(el) {
    return el.is('[data-par]');
  },

  import: function(el, node, converter) {
    var source = el.attr('data-par');
    var matches = source.match("^(\\w+)(\\s+type\\s+(\\w+))?(\\s+default\\s+(.+))?$");
    if(matches){
      node.name = matches[1];
      node.tipe = matches[3];
      node.default = matches[5];
    } else {
      console.error('Error parsing attribute source for StencilParameter: '+source);
      node.name = 'unnamed';
    }
    
    var input = el.find('input');
    if (input) {
      node.value = input.attr('value');
    }

    node.error = el.attr('data-error');

  },

  export: function(node, el, converter) {
    var $$ = converter.$$;

    var source = node.name || 'unnamed';
    if (node.tipe) source += ' type ' + node.tipe;
    if (node.default) source += ' default ' + node.default;
    el.attr('data-par', source);
    
    if (node.value) {
      el.append(
        $$('input')
          .attr('value',node.value)
      );
    }

    if(node.error) el.attr('data-error', node.error);
  }
};
