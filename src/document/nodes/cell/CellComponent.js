import { Component, TextPropertyEditor, findParentComponent } from 'substance'
import TextInput from '../../../shared/substance/text-input/TextInput'
import CodeEditorComponent from '../../ui/CodeEditorComponent'


class CellComponent extends Component {

  didMount() {
    const node = this.props.node
    if (node) {
      node.on('value:updated', this.rerender, this)
    }
  }

  dispose() {
    const node = this.props.node
    if (node) {
      node.off(this)
    }
  }

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
        })
      )
    }
    if (node.value) {
      el.append(
        $$('div').addClass('se-output').text(String(node.valueType)+':'+String(node.value))
      )
    }
    if (node.errors && node.errors.length){
      node.errors.forEach((error) => {
        el.append(
          $$('div').addClass('se-error').text(String(error))
        )
      })
    }
    el.on('click', this._onClick)
    return el
  }

  getExpression() {
    return this.refs.expressionEditor.getContent()
  }

  // HACK: this needs to be replaced with proper utilization of the
  // expression evaluation engine.
  onConfirm() {
    const editorSession = this.context.editorSession
    let newExpression = this.getExpression()
    editorSession.transaction((tx) => {
      tx.set([this.props.node.id, 'expression'], newExpression)
    })
    this.rerender()
  }

  onCancel() {
    this.rerender()
  }

  _onClick(event) {
    let target = findParentComponent(event.target)
    if (target.context.surface) return
    event.stopPropagation()
    this.context.isolatedNodeComponent.selectNode()
  }

}

CellComponent.noBlocker = true

export default CellComponent
