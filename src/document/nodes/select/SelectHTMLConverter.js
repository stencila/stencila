export default {
  type: 'select',
  tagName: 'select',

  matchElement: function (el) {
    return el.is('select')
  },

  import: function (el, node) {
    node.name = el.attr('name')
    node.options = []
    el.findAll('option').forEach(function (option, optionIndex) {
      node.options.push({
        text: option.text(),
        value: option.attr('value')
      })
      if (option.attr('selected')) {
        node.selectedIndex = optionIndex
      }
    })
  },

  export: function (node, el, converter) {
    let $$ = converter.$$
    el.attr('name', node.name)
    node.options.forEach((option, optionIndex) => {
      let optionEl = $$('option')
        .text(option.text)
        .attr('value', option.value)
      if (optionIndex === node.selectedIndex) {
        optionEl.attr('selected', true)
      }
      el.append(optionEl)
    })
  }
}
