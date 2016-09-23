export default {

  type: 'title',
  tagName: 'div',

  matchElement: function (el) {
    return el.is('#title')
  },

  import: function (el, node, converter) {
    node.content = converter.annotatedText(el, [node.id, 'content'])
  },

  export: function (node, el, converter) {
    el.attr('id', 'title')
      .append(converter.annotatedText([node.id, 'content']))
  }
}
