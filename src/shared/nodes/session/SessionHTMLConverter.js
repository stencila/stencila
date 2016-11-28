export default {

  type: 'session',
  tagName: 'div',

  matchElement: function (el) {
    return el.is('.session')
  },

  import: function (el, node, converter) {
    node.url = el.text()
  },

  export: function (node, el, converter) {
    el.addClass('session')
    el.text(node.url)
  }
}
