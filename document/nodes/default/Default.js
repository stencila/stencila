'use strict';

import BlockNode from 'substance/model/BlockNode'

function Default () {
  Default.super.apply(this, arguments);
}

BlockNode.extend(Default);

Default.define({
  type: 'default',
  html: {type: 'string', default: ''}
});

module.exports = Default;
