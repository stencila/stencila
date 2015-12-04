'use strict';

module.exports = {

  type: 'stencil-equation',
  tagName: 'div',

  matchElement: function(el) {
    return el.is('div[data-equation]');
  },

  import: function(el, node, converter) {
    var source = el.find('script');
    if(source.length > 0){
      var format = source.prop('type');
      format = format.split(';')[0];
      node.source = source.text();
      node.format = format;
    }
  },

  export: function(node, el, converter) {
    el
      .attr('data-equation', 'true')
      .append(
        converter.$$('script')
          .prop('type', node.format)
          .text(node.source)
      );
  }
};
