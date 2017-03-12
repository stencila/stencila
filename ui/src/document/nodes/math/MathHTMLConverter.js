export default {

  type: 'math',
  tagName: 'span',

  matchElement: function (el) {
    return el.is('[data-math]')
  },

  import: function (el, node) {
    node.language = el.attr('data-math')
    node.source = el.text()
  },

  export: function (node, el) {
    el.attr('data-math', node.language)
    el.text(node.source)
  }

}
