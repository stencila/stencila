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

  // TODO: This code has not yet been tested
  export: function (node, el, converter) {
    let $$ = converter.$$
    el.attr('data-cell', node.expression)
    el.attr('data-language', node.language)
    el.append(
      $$('pre').attr('data-source', '').text(node.sourceCode)
    )
    el.append(
      $$('pre').attr('data-output', '').text(node.output)
    )

  }

}
