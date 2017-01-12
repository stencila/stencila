export default {

  type: 'execute',
  tagName: 'pre',

  matchElement: function (el) {
    return el.is('pre') && el.attr('data-execute')
  },

  import: function (el, node, converter) {
    node.session = el.attr('data-execute')
    node.input = el.attr('data-input') || ''
    node.output = el.attr('data-output') || ''
    node.show = el.attr('data-show') || false
    node.extra = el.attr('data-extra')
    node.code = el.text()
  },

  export: function (node, el, converter) {
    el.attr('data-execute', node.session)
    if (node.input) el.attr('data-input', node.input)
    if (node.output) el.attr('data-output', node.output)
    if (node.show) el.attr('data-show', node.show)
    if (node.extra) el.attr('data-extra', node.extra)
    el.text(node.code)
  }
}
