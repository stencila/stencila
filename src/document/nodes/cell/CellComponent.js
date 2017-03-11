import { Component, TextPropertyEditor, findParentComponent } from 'substance'
import CodeEditorComponent from '../../ui/CodeEditorComponent'
import CellValueComponent from './CellValueComponent'

class CellComponent extends Component {

  render($$) {
    let node = this.props.node
    let el = $$('div').addClass('sc-cell')
    el.append(
      $$('div').addClass('se-expression').append(
        $$(TextPropertyEditor, {
          path: [node.id, 'expression']
        }).ref('expressionEditor')
      )
    )
    if (node.sourceCode) {
      el.append(
        $$(CodeEditorComponent, {
          node: this.props.node,
          codeProperty: 'sourceCode',
          languageProperty: 'language'
        }).ref('sourceCodeEditor')
          .on('escape', this.onEscapeFromCodeEditor)
      )
    }
    if (node) {
      el.append(
        $$(CellValueComponent, {node}).ref('value')
      )
    }
    el.on('click', this.onClick)
    return el
  }

  getExpression() {
    return this.refs.expressionEditor.getContent()
  }

  onClick(event) {
    let target = findParentComponent(event.target)
    // console.log('###', target, target._owner)
    if (target._owner === this.refs.expressionEditor || target._owner === this.refs.sourceCodeEditor) {
      // console.log('### skipping')
      return
    }
    event.stopPropagation()
    this.context.isolatedNodeComponent.selectNode()
  }

  onEscapeFromCodeEditor(event) {
    event.stopPropagation()
    this.send('escape')
  }

}

CellComponent.noBlocker = true

export default CellComponent
