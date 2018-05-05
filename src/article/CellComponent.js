/* globals clearTimeout */
import { NodeComponent, FontAwesomeIcon, isEqual } from 'substance'
import ValueComponent from '../shared/ValueComponent'
import CodeEditor from '../shared/CodeEditor'
import { getCellState, getError, getErrorMessage } from '../shared/cellHelpers'
import { toString as stateToString, BROKEN, FAILED, OK } from '../engine/CellStates'
import NodeMenu from './NodeMenu'

const LANG_LABELS = {
  'mini': 'Mini',
  'js': 'JS',
  'node': 'Node',
  'sql': 'SQL',
  'py': 'Py',
  'r': 'R',
}

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

  didMount() {
    this.context.editorSession.onRender('document', this._onNodeChange, this, { path: [this.props.node.id]})
  }

  getInitialState() {
    return {
      hideCode: false,
      forceOutput: false
    }
  }

  _renderStatus($$) {
    const cellState = getCellState(this.props.node)
    let statusName = cellState ? stateToString(cellState.status) : 'unknown'
    return $$('div').addClass(`se-status sm-${statusName}`)
  }

  render($$) {
    const cell = this.props.node
    const cellState = getCellState(cell)
    let el = $$('div').addClass('sc-cell')
    el.attr('data-id', cell.id)

    if (!this.state.hideCode) {
      let source = cell.find('source-code')
      let cellEditorContainer = $$('div').addClass('se-cell-editor-container')
      cellEditorContainer.append(
        this._renderStatus($$),
        $$('div').addClass('se-expression').append(
          $$(CodeEditor, {
            path: source.getPath(),
            excludedCommands: this._getBlackListedCommands(),
            language: source.attributes.language,
            multiline: true
          }).ref('expressionEditor')
            .on('escape', this._onEscapeFromCodeEditor)
        )
      )
      el.append(cellEditorContainer)
      el.append(
        this._renderEllipsis($$)
      )
      el.append(
        $$('div').addClass('se-language').append(
          LANG_LABELS[source.attributes.language]
        )
      )
    } else {
      // TODO: Create proper visual style
      el.append(
        $$('button').append(
          this._renderStatus($$),
          $$(FontAwesomeIcon, { icon: 'fa-code' })
        )
          .addClass('se-show-code')
          .attr('title', 'Show Code')
          .on('click', this._showCode)
      )
    }

    if (cellState) {
      const status = cellState.status
      if(status === FAILED || status === BROKEN) {
        let errEl = $$('div').addClass('se-error').append(
          getErrorMessage(getError(cell))
        ).ref('error')
        if (this._hideError) {
          errEl.setStyle('visibility', 'hidden')
        }
        el.append(errEl)
      } else if (status === OK) {
        if (this._showOutput()) {
          el.append(
            $$(ValueComponent, cellState.value).ref('value')
          )
        }
      } else if (this.oldValue && this._showOutput()) {
        el.append(
          $$(ValueComponent, this.oldValue).ref('value')
        ).addClass('sm-pending')
      }
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
    return !this._isDefinition() || this.state.forceOutput
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

  _onNodeChange() {
    const cell = this.props.node
    const cellState = getCellState(cell)
    const oldCellState = this._oldCellState
    if (cellState) {
      // 1. does the cell have an error?
      // 2. did the status or the errors change?
      let showError = true
      const status = cellState.status
      if(status === BROKEN || status === FAILED) {
        if (oldCellState) {
          showError = (
            oldCellState.status !== status ||
            !isEqual(oldCellState.errors.map(e => e.message), cellState.errors.map(e => e.message))
          )
          if (showError) {
            console.log('SHOW ERROR', oldCellState.status, status, oldCellState.errors.join(','), cellState.errors.join(','))
          }
        }
      }
      clearTimeout(this.delayError)
      if (showError) {
        this._hideError = true
        this.delayError = setTimeout(() => {
          const errEl = this.refs.error
          if(errEl) {
            errEl.setStyle('visibility', 'visible')
          }
          this._hideError = false
        }, 500)
      }
      this._oldCellState = {
        status,
        errors: cellState.errors.slice()
      }
      // keep the last valid value to be able to reduce flickering in most of the cases
      if (status === OK) {
        this.oldValue = cellState.value
      }
    }
    this.rerender()
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
    // TODO: not too happy about how difficult it is to set the selection
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
