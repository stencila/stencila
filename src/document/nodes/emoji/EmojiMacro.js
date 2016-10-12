import InlineNodeMacro from '../../ui/InlineNodeMacro'

class EmojiMacro extends InlineNodeMacro {

  get regex () {
    return /:([a-z0-9_]+):/
  }

  createNodeData (match) {
    var name = match[1]
    return {
      type: 'emoji',
      name: name
    }
  }

}

export default EmojiMacro
