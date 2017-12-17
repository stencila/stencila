import { Component } from 'substance'

export default class ContextSection extends Component {

  render($$) {
    const contextId = this.props.contextId

    let el = $$('div').addClass('sc-context-section')

    if (!contextId) {
      console.error('FIXME: could not find contextId')
    } else {
      const ComponentClass = this.getComponent(contextId)

      el.append(
        $$('div').addClass('se-context-header').append(
          $$('div').addClass('se-label').append(this.getLabel(contextId)),
          $$('div').addClass('se-close').append(
            this.context.iconProvider.renderIcon($$, 'context-close')
          ).on('click', this.close)
        ),
        $$('div').addClass('se-context-content').append(
          $$(ComponentClass, {
            cellId: this.props.cellId,
            sectionId: this.props.sectionId
          })
        )
      )
    }

    return el
  }

  close() {
    const contextId = this.props.contextId
    let sheetEditor = this.context.app.getSheetEditor()
    if (sheetEditor) {
      sheetEditor.toggleContext(contextId)
    }
  }
}
