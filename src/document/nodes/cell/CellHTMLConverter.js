export default {

  type: 'cell',
  tagName: 'div',

  matchElement: function (el) {
    return el.is('[data-cell]')
  },

  import: function (el, node) {
    node.language = el.attr('data-language')
    node.expression = el.attr('data-cell')
    let sourceCodeEl = el.find('pre[data-source]')
    if (sourceCodeEl) {
      node.sourceCode = sourceCodeEl.textContent
    }
    let outputEl = el.find('pre[data-output]')
    if (outputEl) {
      node.output = outputEl.innerHTML
    }
  },

  export: function (node, el) {
    el.attr('data-math', node.language)
    el.text(node.source)
  }

}
