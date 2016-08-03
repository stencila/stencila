'use strict';

var Container = require('substance/model/Container');

function Discussion() {
  Discussion.super.apply(this, arguments);
}

Container.extend(Discussion);

Discussion.define({
  type: 'discussion',
  status: { type: 'string', default: 'open' },
  nodes: { type: ['id'], default: [] }
});

module.exports = Discussion;
