import Prism from 'prismjs'
import 'prismjs/plugins/filter-highlight-all/prism-filter-highlight-all'
import 'prismjs/components/prism-python'
import 'prismjs/components/prism-r'
import { ready, select } from '../../util'

ready(() => {
  /**
   * Currently, Encoda erroneously adds itemscope and itemtype attributes
   * to the `<code>` element of a `CodeBlock`:
   *
   * ```html
   * <pre itemscope="" itemtype="http://schema.stenci.la/CodeBlock">
   *   <code itemscope="" itemtype="http://schema.stenci.la/CodeFragment"></code>
   * </pre>
   * ```
   *
   * Instead it should be:
   *
   * ```html
   * <pre itemscope="" itemtype="http://schema.stenci.la/CodeBlock">
   *   <code></code>
   * </pre>
   * ```
   *
   * The offending line is: https://github.com/stencila/encoda/blob/cf09daaa4cfff5bbbf4f9ab2ae786b22f6faa1a5/src/codecs/html/index.ts#L1423
   * which reuses `encodeCodeFragment`
   *
   * This removes the inner `itemscope` and `itemtype`, pending a fix in Encoda.
   */
  select('pre:--CodeBlock > code:--CodeFragment').forEach(element => {
    element.removeAttribute('itemscope')
    element.removeAttribute('itemtype')
  })

  /**
   * Prism does not highlight code unless it has `class=language-xxxx`. This causes
   * `CodeChunk` and `CodeFragment` nodes without a `programmingLanguage` specified
   * to be styled differently. So add these to the list of elements that Prism highlights.
   */
  select('pre:--CodeBlock > code, code:--CodeFragment').forEach(element => {
    if (!element.className.includes('language-'))
      element.classList.add('language-text')
  })

  /**
   * Use https://prismjs.com/plugins/filter-highlight-all/ to reject
   * highlighting any <code> elements that are not part of `CodeFragment` or
   * `CodeBlock` nodes (highlighting of `CodeExpression` and `CodeChunks` nodes
   * is handled by the Web Components for those nodes).
   */
  Prism.plugins.filterHighlightAll.reject.add(
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
