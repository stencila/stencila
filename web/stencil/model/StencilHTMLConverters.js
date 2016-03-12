module.exports = [
  // Substance converters, in alphabetical order, from `substance/packages`
  require('substance/packages/blockquote/BlockquoteHTMLConverter'),
  require('substance/packages/code/CodeHTMLConverter'),
  require('substance/packages/codeblock/CodeblockHTMLConverter'),
  require('substance/packages/emphasis/EmphasisHTMLConverter'),
  require('substance/packages/heading/HeadingHTMLConverter'),
  require('substance/packages/image/ImageHTMLConverter'),
  require('substance/packages/link/LinkHTMLConverter'),
  require('substance/packages/paragraph/ParagraphHTMLConverter'),
  require('substance/packages/strong/StrongHTMLConverter'),
  require('substance/packages/subscript/SubscriptHTMLConverter'),
  require('substance/packages/superscript/SuperscriptHTMLConverter'),
  require('substance/packages/table/TableHTMLConverter'),

  // Stencil-specific converters
  require('../packages/equation/StencilEquationHTMLConverter'),
  require('../packages/exec/StencilExecHTMLConverter'),
  require('../packages/figure/StencilFigureHTMLConverter'),
  require('../packages/math/StencilMathHTMLConverter'),
  require('../packages/parameter/StencilParameterHTMLConverter'),
  require('../packages/summary/StencilSummaryHTMLConverter'),
  require('../packages/text/StencilTextHTMLConverter'),
  require('../packages/title/StencilTitleHTMLConverter')
];
