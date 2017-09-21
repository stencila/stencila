/**
 * A module for rendering of math
 *
 * - uses KaTeX for rendering
 *
 * - uses `ASCIIMathTeXImg.js` for converting ASCIIMath to TeX
 *
 * - in the future may use MathJax as a fallback to any things
 *   that KaTex does not handle (see http://www.intmath.com/blog/mathematics/katex-with-asciimathml-input-and-mathjax-fallback-9456)
 *
 * @module utilities/math
 */

import katex from 'katex'
import am from './asciimath'

/**
 * Translate between math markup languages
 *
 * @param      {string}  markup    The markup
 * @param      {string}  source    The source language ( default 'asciimath')
 * @param      {string}  target    The target language ( default 'tex')
 * @return     {string}  Markup translated to the target language
 */
var translate = function (markup, source, target) {
  source = source || 'asciimath'
  target = target || 'tex'

  if (target === 'tex') {
    if (source === 'tex' || source === 'latex') {
      return markup
    } else if (source === 'am' || source === 'asciimath') {
      return am.toTeX(markup)
    } else {
      throw Error('Unhandled conversion from {' + source + '} to {' + target + '}')
    }
  } else {
    throw Error('Unhandled target language {' + target + '}')
  }
}

/**
 * Render math markup into HTML
 *
 * @param      {string}  markup    The source markup
 * @param      {string}  language  The language ('tex' (default) or 'asciimath') of the source
 * @param      {string}  display   The display mode ('inline' (default) or 'block')
 * @return     {string}  Rendered math HTML
 */
var render = function (markup, language, display) {
  language = language || 'tex'
  display = display || 'inline'

  var tex = translate(markup, language, 'tex')
  return katex.renderToString(tex, {
    displayMode: display === 'block'
  })
}

export default {
  translate: translate,
  render: render
}
