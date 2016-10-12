import EmojiComponent from './EmojiComponent'

class EmojiMarkdownComponent extends EmojiComponent {

  render ($$) {
    var node = this.props.node
    return $$('span')
      .addClass('sc-emoji')
      .text(':' + node.name + ':')
  }

}

export default EmojiMarkdownComponent
