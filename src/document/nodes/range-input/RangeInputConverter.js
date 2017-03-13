export default {
  type: 'range-input',
  tagName: 'input',

  matchElement: function (el) {
    return el.is('input[type="range"]')
  },

  import: function (el, node) {
    node.name = el.attr('name')
    node.min = Number(el.attr('min'))
    node.max = Number(el.attr('max'))
    node.step = Number(el.attr('step'))
    node.value = Number(el.attr('value'))
  },

  export: function (node, el) {
    el.attr('type', 'range')
    el.attr('name', node.name)
    el.attr('min', node.min)
    el.attr('max', node.max)
    el.attr('step', node.step)
    el.attr('value', node.value)
  }
}
