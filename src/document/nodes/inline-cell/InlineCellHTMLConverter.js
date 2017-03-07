export default {

  type: 'inline-cell',
  tagName: 'span',

  matchElement: function (el) {
    return el.is('span[data-cell]')
  },

  import: function (el, node) {
    node.expression = el.attr('data-cell')
    node.output = el.innerHTML
  },

  // TODO: This code has not yet been tested
  export: function (node, el) {
    el.attr('data-cell', node.expression)
    el.innerHTML = node.output
  }

}
