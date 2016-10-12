import Component from 'substance/ui/Component'
import TextPropertyEditor from 'substance/ui/TextPropertyEditor'

class MathMarkdownComponent extends Component {

  render ($$) {
    var node = this.props.node

    var delim
    if (node.language === 'asciimath') {
      delim = '|'
    } else {
      delim = '$'
    }

    return $$('span')
      .addClass('sc-math')
      .append(
        delim,
        $$(TextPropertyEditor, {
          path: [ node.id, 'source' ],
          withoutBreak: true
        }),
        delim
      )
  }

}

export default MathMarkdownComponent
