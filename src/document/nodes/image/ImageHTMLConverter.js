export default {

  type: 'image',
  tagName: 'img',

  matchElement: function (el) {
    return el.is('img')
  },

  import: function (el, node) {
    node.src = el.attr('src')
  },

  export: function (node, el) {
    el.attr('src', node.src)
  }
}
