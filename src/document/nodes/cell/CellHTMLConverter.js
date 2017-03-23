export default {

  type: 'cell',
  tagName: 'div',

  matchElement: function (el) {
    return el.is('div[data-cell]')
  },

  import: function (el, node) {
    const language = el.attr('data-language')
    if (language) {
      node.language = language
    }
    node.expression = el.attr('data-cell')
    let sourceCodeEl = el.find('pre[data-source]')
    if (sourceCodeEl) {
      node.sourceCode = sourceCodeEl.textContent
    }
    let outputEl = el.find('pre[data-output]')
    if (outputEl) {
      node.value = outputEl.innerHTML
    }
  },

  // TODO: This code has not yet been tested
  export: function (node, el, converter) {
    let $$ = converter.$$
    el.attr('data-cell', node.expression)
    if (node.sourceCode) {
      el.attr('data-language', node.language)
      el.append(
        $$('pre').attr('data-source', '').text(node.sourceCode)
      )
    }
    el.append(
      $$('pre').attr('data-output', '').text(node.value)
    )

  }

}
