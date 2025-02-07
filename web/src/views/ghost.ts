// @ts-expect-error 'we a re using a contrib/ module, so the type declarations are a bit busted'
import renderMathInElement from 'katex/dist/contrib/auto-render.mjs'
import 'katex/dist/katex.min.css'

import '../nodes'
import '../shoelace'
import '../ui/document/menu'

document.addEventListener('DOMContentLoaded', () => {
  // intialise katex auto render
  renderMathInElement(document.body, {
    delimiters: [
      { left: '$$', right: '$$', display: true },
      { left: '$', right: '$', display: false },
      { left: '\\(', right: '\\)', display: false },
      { left: '\\[', right: '\\]', display: true },
    ],
  })
})
