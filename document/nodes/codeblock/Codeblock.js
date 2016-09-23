'use strict'

import BlockNode from 'substance/model/BlockNode'

function Codeblock () {
  Codeblock.super.apply(this, arguments)
}

BlockNode.extend(Codeblock)

Codeblock.define({
  type: 'codeblock',
  language: { type: 'string', default: '' },
  source: { type: 'string', default: '' }
})

module.exports = Codeblock

