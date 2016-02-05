module.exports = [
  require('substance/packages/paragraph/ParagraphHTMLConverter'),
  require('substance/packages/heading/HeadingHTMLConverter'),
  require('substance/packages/strong/StrongHTMLConverter'),
  require('substance/packages/emphasis/EmphasisHTMLConverter'),
  require('substance/packages/link/LinkHTMLConverter'),
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
