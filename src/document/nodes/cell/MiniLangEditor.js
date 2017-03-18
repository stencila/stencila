import {Component, TextPropertyEditor} from 'substance'

export default
class MiniLangEditor extends Component {

  render($$) {
    const expression = this.props.expression

    let el = $$('div').addClass('sc-mini-lang-editor')
    // used for showing errors/warnings
    let gutter = $$('div').addClass('se-gutter').ref('gutter')

    if (expression.errors && expression.errors.length>0) {
      gutter.append(
        this.context.iconProvider.renderIcon($$, 'error')
      )
    }

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

    // using a table for layout
    let layout = $$('table').ref('layout')
    layout.append($$('colgroup').append(
      $$('col').addClass('se-gutter-col'),
      $$('col').addClass('se-content-col')
    ))
    layout.append(
      $$('tr').append(
        $$('td').append(gutter),
        $$('td').append(content)
      )
    )
    el.append(layout)
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
          start: {
            path,
            offset: t.start
          },
          end: {
            path,
            offset: t.end
          },
          on(){},
          off(){}
        }
      })
    } else {
      return []
    }
  }

}