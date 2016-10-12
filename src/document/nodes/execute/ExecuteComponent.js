import CodeEditorComponent from '../../ui/CodeEditorComponent'

class ExecuteComponent extends CodeEditorComponent {

  constructor (parent, props) {
    props.codeProperty = 'source'
    props.languageProperty = 'language'
    super(parent, props)
  }

  render ($$) {
    return super.render.call(this, $$)
      .addClass('sc-execute')
  }

}

export default ExecuteComponent
