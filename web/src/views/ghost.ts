import 'katex/dist/katex.min.css'
import renderMathInElement from 'katex/dist/contrib/auto-render.mjs'

import '../nodes'
import '../shoelace'
import '../ui/document/menu'

document.addEventListener('DOMContentLoaded', () => {
  renderMathInElement(document.body, {
    delimiters: [
      { left: '$$', right: '$$', display: true },
      { left: '$', right: '$', display: false },
      { left: '\\(', right: '\\)', display: false },
      { left: '\\[', right: '\\]', display: true },
    ],
  })
})
