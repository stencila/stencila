export default {
  type: 'output',
  tagName: 'output',

  matchElement: function (el) {
    return el.is('output')
  },

  import: function (el, node, converter) {
    node.value = el.attr('for') || null
    node.format = el.attr('data-format') || null
  },

  export: function (node, el, converter) {
    if (node.value) el.attr('for', node.value)
    if (node.format) el.attr('data-format', node.format)
  }
}
