import { Component, BodyScrollPanePackage, Overlay } from 'substance'
import CodeEditor from '../shared/CodeEditor'
const { BodyScrollPane } = BodyScrollPanePackage

export default class FormulaEditor extends Component {

  render($$) {
    let el = $$('div').addClass('sc-formula-editor')
    el.append(this._renderCodeEditor($$, 'formula-editor'))
    return el
  }

  _renderCodeEditor($$, editorId) {
    const node = this.props.context.node
    const configurator = this.props.context.configurator
    let scrollPane = this._renderScrollPane($$)
    return scrollPane.append(
      $$(CodeEditor, {
        name: editorId,
        path: node.getPath(),
        multiline: false,
        mode: 'cell',
        language: this.props.language
      }).ref('cellEditor'),
      $$(Overlay, {
        toolPanel: configurator.getToolPanel('prompt'),
        theme: 'dark'
      }).ref('overlay')
    )
  }

  _renderScrollPane($$) {
    return $$(BodyScrollPaneForSheetComponent).ref('scrollPane')
  }

  getChildContext() {
    return this.props.context
  }

  getSurfaceId() {
    return this.refs.cellEditor.getSurfaceId()
  }

}

class BodyScrollPaneForSheetComponent extends BodyScrollPane {

  getContentElement() {
    return this.getElement()
  }

}