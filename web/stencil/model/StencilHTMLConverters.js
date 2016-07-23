module.exports = [
  // Substance converters, in alphabetical order, from `substance/packages`
  require('substance-fe0ed/packages/blockquote/BlockquoteHTMLConverter'),
  require('substance-fe0ed/packages/code/CodeHTMLConverter'),
  //require('substance-fe0ed/packages/codeblock/CodeblockHTMLConverter'),
  require('substance-fe0ed/packages/emphasis/EmphasisHTMLConverter'),
  require('substance-fe0ed/packages/heading/HeadingHTMLConverter'),
  require('substance-fe0ed/packages/image/ImageHTMLConverter'),
  require('substance-fe0ed/packages/link/LinkHTMLConverter'),
  require('substance-fe0ed/packages/paragraph/ParagraphHTMLConverter'),
  require('substance-fe0ed/packages/strong/StrongHTMLConverter'),
  require('substance-fe0ed/packages/subscript/SubscriptHTMLConverter'),
  require('substance-fe0ed/packages/superscript/SuperscriptHTMLConverter'),
  require('substance-fe0ed/packages/table/TableHTMLConverter'),

  // Stencil-specific converters
  require('../packages/codeblock/StencilCodeblockHTMLConverter'),
  require('../packages/equation/StencilEquationHTMLConverter'),
  require('../packages/exec/StencilExecHTMLConverter'),
  require('../packages/figure/StencilFigureHTMLConverter'),
  require('../packages/include/StencilIncludeHTMLConverter'),
  require('../packages/math/StencilMathHTMLConverter'),
  require('../packages/out/StencilOutHTMLConverter'),
  require('../packages/parameter/StencilParameterHTMLConverter'),
  require('../packages/summary/StencilSummaryHTMLConverter'),
  require('../packages/text/StencilTextHTMLConverter'),
  require('../packages/title/StencilTitleHTMLConverter')
];
