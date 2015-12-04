module.exports = [
  require('substance/packages/paragraph/ParagraphHTMLConverter'),
  require('substance/packages/heading/HeadingHTMLConverter'),
  require('substance/packages/strong/StrongHTMLConverter'),
  require('substance/packages/emphasis/EmphasisHTMLConverter'),
  require('substance/packages/link/LinkHTMLConverter'),

  // Stencil-specific converters
  require('../packages/exec/StencilExecHTMLConverter'),
];
