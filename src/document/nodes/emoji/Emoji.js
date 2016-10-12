import InlineNode from 'substance/model/InlineNode'

class Emoji extends InlineNode {}

Emoji.define({
  type: 'emoji',
  name: { type: 'string' }
})

export default Emoji
