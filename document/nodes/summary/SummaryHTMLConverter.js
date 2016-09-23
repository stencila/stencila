'use strict'

module.exports = {

  type: 'summary',
  tagName: 'div',

  matchElement: function (el) {
    return el.is('#summary')
  },

  import: function (el, node, converter) {
    node.content = converter.annotatedText(el, [node.id, 'content'])
  },

  export: function (node, el, converter) {
    el.attr('id', 'summary')
      .append(converter.annotatedText([node.id, 'content']))
  }
}
