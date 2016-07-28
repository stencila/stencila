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
 * @module shared/math
 */

var katex = require('katex');
require('./ASCIIMathTeXImg');

module.exports = {
    
  /**
   * Render math markup into HTML
   *
   * @param      {string}  source    The source markup 
   * @param      {string}  language  The language ('tex' or 'asciimath')
   * @return     {string}  Rendered math HTML
   */
  render: function(source, language) {
    language = language || 'tex';
    var tex;
    if (language === 'tex' || language === 'latex') {
      tex = source;
    } else {
      tex = window.AMTparseAMtoTeX(source);
    }
    return katex.renderToString(tex);
  }

};
