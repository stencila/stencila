import {Component} from 'substance'

import CodeEditorComponent from '../../ui/CodeEditorComponent'

class DefaultMarkdownComponent extends Component {

  render ($$) {
    var node = this.props.node
    return super.render.call(this, $$)
      .addClass('sc-default')
      .append(
        $$(CodeEditorComponent, {
          node: node,
          codeProperty: 'html',
          languageProperty: null,
          language: 'html'
        }).ref('code')
      )
  }

}

DefaultMarkdownComponent.fullWidth = true

export default DefaultMarkdownComponent
