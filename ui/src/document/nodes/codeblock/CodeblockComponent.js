import CodeEditorComponent from '../../ui/CodeEditorComponent'

class CodeblockComponent extends CodeEditorComponent {

  constructor (parent, props) {
    props.codeProperty = 'source'
    props.languageProperty = 'language'
    super(parent, props)
  }

  render ($$) {
    return super.render.call(this, $$)
      .addClass('sc-codeblock')
  }

}

CodeblockComponent.fullWidth = true

export default CodeblockComponent
