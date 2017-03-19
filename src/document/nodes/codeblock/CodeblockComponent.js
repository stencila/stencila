import CodeEditorComponent from '../../ui/CodeEditorComponent'

class CodeblockComponent extends CodeEditorComponent {

  constructor (parent, props) {
    super(parent, Object.assign(props, {
      path: [props.node.id, 'source'],
      language: props.node.language
    }))
  }

  didMount() {
    super.didMount()

    const editorSession = this.context.editorSession
    const node = this.props.node
    editorSession.on('render', this._onLanguageChanged, this, {
      resource: 'document',
      path: [node.id, this.props.languageProperty]
    })
  }

  dispose() {
    super.dispose()

    const editorSession = this.context.editorSession
    editorSession.off(this)
  }

  render($$) {
    return super.render($$)
      .addClass('sc-codeblock')
  }

  _onLanguageChanged() {
    super.setLanguage(this.props.node.language)
  }

}

CodeblockComponent.fullWidth = true

export default CodeblockComponent
