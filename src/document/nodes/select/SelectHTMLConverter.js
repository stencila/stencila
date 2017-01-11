export default {
  type: 'select',
  tagName: 'select',

  matchElement: function (el) {
    return el.is('select')
  },

  import: function (el, node, converter) {
    node.name = el.attr('name')
    el.findAll('option').forEach(function (option) {
      node.options.push(`${option.val()}\t${option.text()}`)
    })
    node.value = el.val()
  },

  export: function (node, el, converter) {
    el.attr('name', node.name)
  }
}
