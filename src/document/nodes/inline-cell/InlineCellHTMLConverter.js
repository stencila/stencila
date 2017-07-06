export default {
  type: 'inline-cell',
  tagName: 'output',

  matchElement: function (el) {
    return el.is('output[for]')
  },

  import: function (el, node) {
    node.expression = el.attr('for')
    node.value = el.innerHTML
  },

  export: function (node, el) {
    el.attr('for', node.expression)
    el.innerHTML = node.value
  }
}
