import Prism from 'prismjs'
import 'prismjs/plugins/filter-highlight-all/prism-filter-highlight-all'
import 'prismjs/components/prism-python'
import 'prismjs/components/prism-r'
import { ready, select } from '../../util'
import { PrismJsPlugins } from '../../libs'

ready(() => {
  /**
   * Prism does not highlight code unless it has `class=language-xxxx`. This causes
   * `CodeChunk` and `CodeFragment` nodes without a `programmingLanguage` specified
   * to be styled differently. So add these to the list of elements that Prism highlights.
   */
  select('pre:--CodeBlock > code, code:--CodeFragment').forEach((element) => {
    if (!element.className.includes('language-'))
      element.classList.add('language-text')
  })

  /**
   * Use https://prismjs.com/plugins/filter-highlight-all/ to reject
   * highlighting any <code> elements that are not part of `CodeFragment` or
   * `CodeBlock` nodes (highlighting of `CodeExpression` and `CodeChunks` nodes
   * is handled by the Web Components for those nodes).
   */
  const plugins = Prism.plugins as PrismJsPlugins

  plugins.filterHighlightAll.reject.add(
    (code: { element: Element; language: string }) => {
      const { element } = code
      const itemtype = element.getAttribute('itemtype')
      if (itemtype === 'http://schema.stenci.la/CodeFragment') return false
      if (
        element.parentElement?.getAttribute('itemtype') ===
        'http://schema.stenci.la/CodeBlock'
      )
        return false
      return true
    }
  )
})
