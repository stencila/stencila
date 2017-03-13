export default {

  type: 'include',
  tagName: 'div',

  matchElement: function (el) {
    return el.is('[data-include]')
  },

  import: function (el, node) {
    node.address = el.attr('data-include')
    node.selector = el.attr('data-selector') || ''
    node.input = el.attr('data-input') || ''
  },

  export: function (node, el) {
    el.attr('data-include', node.address)
    if (node.selector) el.attr('data-selector', node.selector)
    if (node.input) el.attr('data-input', node.input)
  }
}
