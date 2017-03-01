import { Component, TextPropertyEditor } from 'substance'

class ImageMarkdownComponent extends Component {

  render ($$) {
    var node = this.props.node
    return $$('span')
      .addClass('sc-image')
      .append(
        '![](',
        $$(TextPropertyEditor, {
          path: [ node.id, 'src' ],
          withoutBreak: true
        }),
        ')'
      )
  }

}

export default ImageMarkdownComponent
