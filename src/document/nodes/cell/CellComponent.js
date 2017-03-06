import { Component } from 'substance'
import TextInput from '../../../shared/substance/text-input/TextInput'
import CodeEditorComponent from '../../ui/CodeEditorComponent'


let exampleOutput = `
<style>
.chart div {
  font: 10px sans-serif;
  background-color: steelblue;
  text-align: right;
  padding: 3px;
  margin: 1px;
  color: white;
}
</style>
<div class="chart">
  <div style="width: 40px;">4</div>
  <div style="width: 80px;">8</div>
  <div style="width: 150px;">15</div>
  <div style="width: 160px;">16</div>
  <div style="width: 230px;">23</div>
  <div style="width: 420px;">42</div>
</div>`

class CellComponent extends Component {

  render($$) {
    let node = this.props.node
    let el = $$('div').addClass('sc-cell')
    el.append(
      $$('div').addClass('se-expression').append(
        $$(TextInput, {
          content: node.expression
        }).ref('expressionEditor')
          .on('confirm', this.onConfirm)
          .on('cancel', this.onCancel)
      )
    )
    if (node.sourceCode) {
      // props.codeProperty = 'source'
      // props.languageProperty = 'language'

      el.append(
        $$(CodeEditorComponent, {
          node: this.props.node,
          codeProperty: 'sourceCode',
          languageProperty: 'language'
        })
        // $$('pre').append(
        //   node.sourceCode
        // )
      )
    }
    if (node.output) {
      el.append(
        $$('div').addClass('se-output').html(node.output)
      )
    }
    return el
  }

  getExpression() {
    return this.refs.expressionEditor.getContent()
  }

  // HACK: this needs to be replaced with proper utilization of the
  // expression evaluation engine.
  onConfirm() {
    let newExpression = this.getExpression()
    this.context.editorSession.transaction((tx) => {
      tx.set([this.props.node.id, 'expression'], newExpression)
      tx.set([this.props.node.id, 'output'], exampleOutput)
    })
    this.rerender()
  }

  onCancel() {
    this.rerender()
  }

}

export default CellComponent
