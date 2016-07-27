'use strict';

var InlineNode = require('substance/model/InlineNode');

function Print() {
  Print.super.apply(this, arguments);
}

InlineNode.extend(Print);

Print.define({
  type: 'print',
  source: { type: 'string', default: '' },
  status: { type: 'string', default: '' },
  error: { type: 'string', optional: true },
  content: { type: 'string', default: '' }
});

module.exports = Print;
