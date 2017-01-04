export default {

  type: 'execute',
  tagName: 'pre',

  matchElement: function (el) {
    return el.is('pre') && el.attr('data-execute')
  },

  import: function (el, node, converter) {
    node.session = el.attr('data-execute')
    node.inputs = el.attr('data-input')
    node.output = el.attr('data-output')
    node.errors = el.attr('data-error')
    node.extra = el.attr('data-extra')
    node.code = el.text()
  },

  export: function (node, el, converter) {
    el.attr('data-execute', node.session)

    /*
    if (node.inputs) {
      el.attr('data-error', node.error)
    }

    if (node.extra) {
      el.attr('data-extra', node.extra)
    }
    */

    el.text(node.code)
  }
}
