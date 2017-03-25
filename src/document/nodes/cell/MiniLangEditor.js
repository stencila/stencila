import { Component, TextPropertyEditor } from 'substance'

export default
class MiniLangEditor extends Component {

  render($$) {
    let el = $$('div').addClass('sc-mini-lang-editor')

    // the source code
    const path = this.props.path
    const commands = this.props.commands
    const markers = this._getMarkers()
    let content = $$(TextPropertyEditor, {
      path,
      commands,
      markers
    }).ref('contentEditor')
    content.addClass('se-content')
    el.append(content)
    return el
  }

  _getMarkers() {
    const expression = this.props.expression
    const path = this.props.path
    if (expression) {
      return expression.tokens.map((t) => {
        return {
          type: 'code-highlight',
          name: t.type,
          start: { path, offset: t.start },
          end: { path, offset: t.end },
          on() {},
          off() {}
        }
      })
    } else {
      return []
    }
  }

}
