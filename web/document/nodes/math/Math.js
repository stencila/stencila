'use strict';

var InlineNode = require('substance/model/InlineNode');

function Math () {
  Math.super.apply(this, arguments);
}

InlineNode.extend(Math);

Math.define({
  type: 'math',
  source: { type: 'string', default: '' },
  language: { type: 'string', default: 'asciimath' },
  display: { type: 'string', default: 'inline' },
  error: { type: 'string', optional: true }
});

module.exports = Math;
