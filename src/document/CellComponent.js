import { NodeComponent, Menu } from 'substance'
import CellValueComponent from '../shared/CellValueComponent'
import CellErrorDisplay from '../shared/CellErrorDisplay'
import MiniLangEditor from '../shared/MiniLangEditor'


export default class CellComponent extends NodeComponent {

  getAppStateSlot() {
    return this.props.node.id
  }

  constructor(...args) {
    super(...args)

    this.handleActions({
      // triggered by MiniLangEditor
      'execute': this._onExecute,
      'break': this._onBreak
    })
  }

  getInitialState() {
    return {
      hideCode: false,
      forceOutput: false
    }
  }

  render($$) {
    const engine = this.context.cellEngine
    const cell = this.props.node
    const cellId = cell.id
    let el = $$('div').addClass('sc-cell')

    if (!this.state.hideCode) {
      let expr = engine.getExpression(cellId)
      let source = cell.find('source-code')
      let cellEditorContainer = $$('div').addClass('se-cell-editor-container')
      cellEditorContainer.append(
        $$('div').addClass('se-expression').append(
          $$(MiniLangEditor, {
            path: source.getPath(),
            excludedCommands: this._getBlackListedCommands(),
            expression: expr
          }).ref('expressionEditor')
            .on('escape', this._onEscapeFromCodeEditor)
        )
      )
      el.append(cellEditorContainer)
      el.append(
        $$(CellErrorDisplay, {cell})
      )
    } else {
      // TODO: Create proper visual style
      el.append(
        $$('button').append('Show Code')
      )
    }

    el.append(
      this._renderEllipsis($$)
    )

    if (this._showOutput()) {
      el.append(
        $$(CellValueComponent, {cell}).ref('value')
      )
    }
    return el
  }

  getExpression() {
    return this.refs.expressionEditor.getContent()
  }

  _toggleMenu() {
    this.context.appState.set({
      'activePopup':  this.refs.menu.props.id,
      'popupType': 'cell-menu'
    })
  }

  /*
    Move this into an overlay, shown depending on app state
  */
  _renderEllipsis($$) {
    let Button = this.getComponent('button')
    let el = $$('div').addClass('se-ellipsis')

    let button = $$(Button, {
      icon: 'ellipsis',
      active: false,
      theme: 'light'
    }).on('click', this._toggleMenu)
    el.append(button)
    // TODO: show menu only when cell is selected (node selection)
    el.append(
      $$(Menu, {
        items: [
          { command: 'hide-cell-code' },
          { command: 'show-cell-code' },
          // { command: 'force-cell-output' },
          // { command: 'unforce-cell-output' },
          { command: 'set-mini' },
          { command: 'set-js' },
          { command: 'set-py' },
          { command: 'set-r' }
        ]
      }).ref('menu')
    )
    return el
  }

  _getBlackListedCommands() {
    const commandGroups = this.context.commandGroups
    let result = []
    ;['annotations', 'insert', 'prompt', 'text-types'].forEach((name) => {
      if (commandGroups[name]) {
        result = result.concat(commandGroups[name])
      }
    })
    return result
  }

  /*
    Generally output is shown when cell is not a definition, however it can be
    enforced
  */
  _showOutput() {
    return !this._isDefinition() || this.state.forceOutput
  }

  _isDefinition() {
    this.context.cellEngine.isDefinition(this.props.node.id)
  }


  _onExecute() {
    this.context.cellEngine.recompute(this.props.node.id)
  }

  _onBreak() {
    this.context.editorSession.transaction((tx) => {
      tx.selection = this._afterNode()
      tx.insertBlockNode({
        type: 'p'
      })
    })
  }

  _onEscapeFromCodeEditor(event) {
    event.stopPropagation()
    this.send('escape')
  }

  _afterNode() {
    // TODO: not too happy about how difficult it is
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
