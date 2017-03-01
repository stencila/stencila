import { InlineNode } from 'substance'

class Emoji extends InlineNode {}

Emoji.define({
  type: 'emoji',
  name: { type: 'string' }
})

export default Emoji
