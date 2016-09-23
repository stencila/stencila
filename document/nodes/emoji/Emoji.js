'use strict'

import InlineNode from 'substance/model/InlineNode'

function Emoji () {
  Emoji.super.apply(this, arguments)
}

InlineNode.extend(Emoji)

Emoji.define({
  type: 'emoji',
  name: { type: 'string' }
})

export default Emoji
