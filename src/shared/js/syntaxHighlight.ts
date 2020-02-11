import Prism from 'prismjs'
import 'prismjs/components/prism-javascript'
import 'prismjs/components/prism-json'
import 'prismjs/components/prism-markdown'
import 'prismjs/components/prism-markup'
import 'prismjs/components/prism-python'
import 'prismjs/components/prism-r'
import 'prismjs/plugins/line-highlight/prism-line-highlight'
import 'prismjs/plugins/line-numbers/prism-line-numbers'

export const codeHighlight = (): void => {
  document
    .querySelectorAll('pre[class*="language-"]')
    .forEach(node => node.classList.add('line-numbers'))

  Prism.highlightAll()
}
