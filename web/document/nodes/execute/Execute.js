'use strict';

var BlockNode = require('substance/model/BlockNode');

function Execute () {

  Execute.super.apply(this, arguments);

}

BlockNode.extend(Execute);

Execute.define({
  type: 'execute',
  language: { type: 'string', default: '' },
  show: { type: 'boolean', default: false },
  error: { type: 'string', optional: true },
  extra: { type: 'string', optional: true },
  source: { type: 'string', default: '' }
});

module.exports = Execute;
