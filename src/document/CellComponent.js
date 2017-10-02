import { NodeComponent } from 'substance'
import CodeEditorComponent from '../shared/CodeEditorComponent'
import CellValueComponent from '../shared/CellValueComponent'
import CellErrorDisplay from '../shared/CellErrorDisplay'
import MiniLangEditor from '../shared/MiniLangEditor'

import { findMini, findSource } from './cellHelpers'

export default
class CellComponent extends NodeComponent {

  constructor(...args) {
    super(...args)

    this.handleActions({
      // triggered by CodeEditorComponent and MiniLangEditor
      'execute': this._onExecute,
      'break': this._onBreak
    })
  }

  getInitialState() {
    return {
      showMenu: false,
      showCode: true,
      forceShowOutput: false
    }
  }

  render($$) {
    const engine = this.context.cellEngine
    const cell = this.props.node
    const cellId = cell.id
    let el = $$('div').addClass('sc-cell')

    let toggleCell = $$('div').addClass('se-toggle-cell').append(
      $$('div').addClass('se-toggle-cell-inner')
    ).on('click', this._toggleMenu)

    if (this.state.showMenu) {
      toggleCell.append(
        this._renderMenu($$)
      )
    }

    if (this.state.showCode) {
      toggleCell.addClass('sm-code-shown')
    }

    if (engine.hasErrors(cellId)) {
      toggleCell.addClass('sm-has-errors')
    }
    el.append(toggleCell)

    if (this.state.showCode) {
      let expr = engine.getExpression(cellId)
      let mini = findMini(cell)
      let source = findSource(cell)
      let cellEditorContainer = $$('div').addClass('se-cell-editor-container')
      cellEditorContainer.append(
        $$('div').addClass('se-expression').append(
          $$(MiniLangEditor, {
            path: mini.getPath(),
            excludedCommands: this._getBlackListedCommands(),
            expression: expr
          }).ref('expressionEditor')
            .on('escape', this._onEscapeFromCodeEditor)
        )
      )
      if (expr && expr.external) {
        cellEditorContainer.append(
          $$(CodeEditorComponent, {
            path: source.getPath(),
            language: expr.context
          }).ref('sourceCodeEditor')
            .on('escape', this._onEscapeFromCodeEditor)
        )
      }
      el.append(cellEditorContainer)
      el.append(
        $$(CellErrorDisplay, {cell})
      )
    }

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

  _renderMenu($$) {
    let menuEl = $$('div').addClass('se-menu')
    menuEl.append(
      this._renderToggleCode($$),
      this._renderToggleOutput($$)
    )
    return menuEl
  }

  /*
    Displays 'Show Code' or 'Hide Code' depending on the current state
  */
  _renderToggleCode($$) {
    let el = $$('div')
      .addClass('se-menu-item')
      .on('click', this._toggleShowCode)

    if (this.state.showCode) {
      el.append('Hide Code')
    } else {
      el.append('Show Code')
    }
    return el
  }

  _renderToggleOutput($$) {
    let el = $$('div')
      .addClass('se-menu-item')
      .on('click', this._toggleForceShowOutput)

    // If cell is not a definition we ensure output is always shown
    if (!this._isDefinition()) {
      el.addClass('sm-disabled')
    }
    if (this._showOutput()) {
      el.append('Hide Output')
    } else {
      el.append('Show Output')
    }
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

  _toggleShowCode(event) {
    event.preventDefault()
    event.stopPropagation()
    this.extendState({
      showCode: !this.state.showCode,
      showMenu: false
    })
  }

  _toggleForceShowOutput(event) {
    event.preventDefault()
    event.stopPropagation()
    // No toggling allowed if cell is not a definition
    if (!this._isDefinition()) return
    this.extendState({
      forceShowOutput: !this.state.forceShowOutput,
      showMenu: false
    })
  }

  /*
    Generally output is shown when cell is not a definition, however it can be
    enforced
  */
  _showOutput() {
    return !this._isDefinition() || this.state.forceShowOutput
  }

  _isDefinition() {
    this.context.cellEngine.isDefinition(this.props.node.id)
  }

  _toggleMenu() {
    this.extendState({
      showMenu: !this.state.showMenu
    })
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
