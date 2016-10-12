import Component from 'substance/ui/Component'

import emojione from 'emojione'
// Consistent with making everying served locally (for offline use etc)...
if (typeof window !== 'undefined') {
  emojione.imagePathPNG = (window.stencila.root || '/web') + '/emojione/png/'
}

class EmojiComponent extends Component {

  didMount () {
    this.props.node.on('name:changed', this.rerender, this)
  }

  dispose () {
    this.props.node.off(this)
  }

  render ($$) {
    var node = this.props.node
    var el = $$('span')
      .addClass('sc-emoji')
    var shortname = ':' + node.name + ':'
    var img = emojione.shortnameToImage(shortname)
    if (img === shortname) {
      // Emoji name is not matched. Indicate this
      // but show name to reflect user intent
      el.addClass('sm-unknown')
        .text(shortname)
    } else {
      // Emoji found so append `img` tag produced by EmojiOne
      el.html(img)
    }
    return el
  }
}

export default EmojiComponent
