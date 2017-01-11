export default {
  type: 'input',
  tagName: 'input',

  matchElement: function (el) {
    return el.is('input')
  },

  import: function (el, node, converter) {
    node.name = el.attr('name')
    node.type_ = el.attr('type') || 'text'
    node.value = el.attr('value') || null
  },

  export: function (node, el, converter) {
    el.attr({
      name: node.name,
      type: node.type_
    })
    if (node.value) el.attr('value', node.value)
  }
}
