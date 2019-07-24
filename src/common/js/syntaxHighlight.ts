import Prism from 'prismjs'
import 'prismjs/components/prism-json'
import 'prismjs/components/prism-markdown'
import 'prismjs/plugins/line-highlight/prism-line-highlight'
import 'prismjs/plugins/line-numbers/prism-line-numbers'

const ready = (): void => {
  document
    .querySelectorAll('pre[class*="language-"]')
    .forEach(node => node.classList.add('line-numbers'))

  Prism.highlightAll()
}

document.addEventListener('DOMContentLoaded', ready)
