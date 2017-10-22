import { NodeComponent, FontAwesomeIcon } from 'substance'
import CellValueComponent from '../shared/CellValueComponent'
import CellErrorDisplay from '../shared/CellErrorDisplay'
import MiniLangEditor from '../shared/MiniLangEditor'
import { getCellState } from '../shared/cellHelpers'
import NodeMenu from './NodeMenu'

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
      hideCode: false,
      forceOutput: false
    }
  }

  render($$) {
    //console.log('cell rerender', this.props.node.id, this.state.hideCode)
    const cell = this.props.node
    const cellState = getCellState(cell)
    let el = $$('div').addClass('sc-cell')

    if (!this.state.hideCode) {
      let source = cell.find('source-code')
      let cellEditorContainer = $$('div').addClass('se-cell-editor-container')
      cellEditorContainer.append(
        $$('div').addClass('se-expression').append(
          $$(MiniLangEditor, {
            path: source.getPath(),
            excludedCommands: this._getBlackListedCommands(),
            tokens: cellState.tokens
          }).ref('expressionEditor')
            .on('escape', this._onEscapeFromCodeEditor)
        )
      )
      el.append(cellEditorContainer)
      el.append(
        $$(CellErrorDisplay, {cell})
      )

      el.append(
        this._renderEllipsis($$)
      )
    } else {
      // TODO: Create proper visual style
      el.append(
        $$('button').append(
          $$(FontAwesomeIcon, { icon: 'fa-code' })
        )
          .addClass('se-show-code')
          .attr('title', 'Show Code')
          .on('click', this._showCode)
      )
    }

    if (this._showOutput()) {
      el.append(
        $$(CellValueComponent, {cell}).ref('value')
      )
    }
    return el
  }

  /*
    Move this into an overlay, shown depending on app state
  */
  _renderEllipsis($$) {
    let Button = this.getComponent('button')
    let el = $$('div').addClass('se-ellipsis')
    let configurator = this.context.editorSession.getConfigurator()
    let button = $$(Button, {
      icon: 'ellipsis',
      active: false,
      theme: 'light'
    }).on('click', this._toggleMenu)
    el.append(button)

    let sel = this.context.editorSession.getSelection()
    if (sel.isNodeSelection() && sel.getNodeId() === this.props.node.id) {
      el.append(
        $$(NodeMenu, {
          toolPanel: configurator.getToolPanel('node-menu')
        }).ref('menu')
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

  _showCode() {
    this.extendState({
      hideCode: false
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
    const cellState = getCellState(this.props.node)
    return cellState && cellState.hasOutput()
  }

  _toggleMenu() {
    this.context.editorSession.setSelection({
      type: 'node',
      containerId: 'body-content-1',
      surfaceId: 'bodyEditor',
      nodeId: this.props.node.id,
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
