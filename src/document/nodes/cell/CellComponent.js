import { Component, parseKeyEvent } from 'substance'
import CodeEditorComponent from '../../ui/CodeEditorComponent'
import CellValueComponent from './CellValueComponent'
import MiniLangEditor from './MiniLangEditor'
import CellErrorDisplay from './CellErrorDisplay'
import Dropdown from '../../../shared/Dropdown'

const LANGUAGE_LABELS = {
  'js': 'JavaScript',
  'javascript': 'JavaScript'
}

class CellComponent extends Component {

  getInitialState() {
    return {
      visibility: 'auto'
    }
  }

  didMount() {
    const node = this.props.node
    const editorSession = this.context.editorSession
    editorSession.on('render', this.onCellChanged, this, {
      resource: 'document',
      path: [node.id]
    })
  }

  dispose() {
    const editorSession = this.context.editorSession
    editorSession.off(this)
  }

  render($$) {
    const cell = this.props.node
    let el = $$('div').addClass('sc-cell')

    let cellEditorContainer = $$('div').addClass('se-cell-editor-container')
    cellEditorContainer.append(
      $$('div').addClass('se-expression').append(
        $$(MiniLangEditor, {
          path: [cell.id, 'expression'],
          commands: ['undo', 'redo', 'select-all'],
          expression: cell.getExpressionNode()
        }).ref('expressionEditor')
          .on('enter', this.onExpressionEnter)
      )
    )
    cellEditorContainer.append(
      this.renderEllipsesDropdown($$)
    )

    if (cell.isExternal()) {
      cellEditorContainer.append(
        $$(CodeEditorComponent, {
          path: [cell.id, 'sourceCode'],
          language: cell.language
        }).ref('sourceCodeEditor')
          .on('escape', this.onEscapeFromCodeEditor)
      )

      el.append(
        $$('div').addClass('se-language-label').append(
          LANGUAGE_LABELS[cell.language]
        )
      )
    }

    el.append(cellEditorContainer)

    // TODO only show the node value if
    // either the node is not assigning to a variable
    // or the user has chosen to show the output
    let showValue = false
    switch(this.state.visibility) {
      case 'hidden':
        showValue = false
        break
      case 'show':
        showValue = true
        break
      // 'auto' is the default
      case 'auto': // eslint-disable-line no-fallthrough
      default:
        showValue = !cell.isDefinition()
        break
    }
    if (showValue) {
      el.append(
        $$(CellValueComponent, {cell})
        .ref('value')
      )
    }
    el.append(
      $$(CellErrorDisplay, {cell})
    )
    return el
  }

  renderEllipsesDropdown($$) {
    // TODO: please feel free to change anything of this mechanism
    const el = $$(Dropdown, {
      icon: 'ellipsis',
      items: [
        {
          type: 'choice',
          label: 'Visibility',
          name: 'visibility',
          value: this.state.visibility,
          choices: [{
            label: 'Auto',
            value: 'auto'
          }, {
            label: 'Show',
            value: 'show'
          }, {
            label: 'Hide',
            value: 'hidden'
          }]
        }
      ]
    }).addClass('se-ellipses')
      .on('select', this.onSelectEllipsesDropdown)
    return el
  }

  getExpression() {
    return this.refs.expressionEditor.getContent()
  }

  onEscapeFromCodeEditor(event) {
    event.stopPropagation()
    this.send('escape')
  }

  onExpressionEnter(event) {
    // EXPERIMENTAL: we want an easy way to insert content after the
    const data = event.detail
    const editorSession = this.context.editorSession
    const modifiers = parseKeyEvent(data, 'modifiers-only')
    switch(modifiers) {
      case 'ALT': {
        editorSession.setSelection(this._afterNode())
        editorSession.executeCommand('insert-cell')
        break
      }
      case 'CTRL': {
        this.props.node.recompute()
        break
      }
      case 'SHIFT': {
        editorSession.transaction((tx) => {
          tx.insertText('\n')
        })
        break
      }
      case '': {
        editorSession.setSelection(this._afterNode())
        editorSession.executeCommand('insert-node', {type:'paragraph'})
        break
      }
      default:
        //
    }
  }

  onCellChanged() {
    this.rerender()
  }

  onSelectEllipsesDropdown(event) {
    const data = event.detail
    const { name, value } = data
    if (name) {
      let newState = {}
      newState[name] = value
      this.extendState(newState)
    } else {
      console.error('FIXME: illegal event emitted by Dropdown')
    }
  }

  _afterNode() {
    // HACK: not too happy about how difficult it is
    // to set the selection
    const node = this.props.node
    const isolatedNode = this.context.isolatedNodeComponent
    const parentSurface = isolatedNode.getParentSurface()
    return {
      type: 'node',
      nodeId: node.id,
      mode: 'after',
      containerId: parentSurface.getContainerId(),
      surfaceId: parentSurface.id
    }
  }
}

CellComponent.noBlocker = true

export default CellComponent
