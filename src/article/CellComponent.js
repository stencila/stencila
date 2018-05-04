import { NodeComponent } from 'substance'
import ValueComponent from '../shared/ValueComponent'
import CodeEditor from '../shared/CodeEditor'
import { getCellState, getError } from '../shared/cellHelpers'
import { toString as stateToString, BROKEN, FAILED, OK, READY } from '../engine/CellStates'
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
    }

    if (cellState) {
      const status = cellState.status
      if(status === FAILED || status === BROKEN) {
        el.append(
          $$('div').addClass('se-error').append(
            getError(cell).message
          ).ref('error').setStyle('visibility', 'hidden')
        )
      } else if (status === OK) {
        if (this._showOutput()) {
          el.append(
            $$(ValueComponent, cellState.value).ref('value')
          )
        }
      } else if (this.oldValue) {
        el.addClass('sm-pending')
        if(this.oldValue.error) {
          el.append(
            $$('div').addClass('se-error').append(
              this.oldValue.error
            ).ref('error').setStyle('visibility', 'hidden')
          )
        } else if (this._showOutput()) {
          el.append(
            $$(ValueComponent, this.oldValue).ref('value')
          )
        }
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
    if(cellState) {
      const status = cellState.status
      if(status === BROKEN || status === FAILED) {
        this.oldValue = {
          error: getError(cell).message
        }
        clearTimeout(this.delayError) // eslint-disable-line no-undef
        this.delayError = setTimeout(() => {
          const errEl = this.refs.error
          if(errEl) {
            errEl.setStyle('visibility', 'visible')
          }
        }, 500)
      } else if (status === OK) {
        this.oldValue = cellState.value
        clearTimeout(this.delayError) // eslint-disable-line no-undef
      } else if (status === READY) {
        clearTimeout(this.delayError) // eslint-disable-line no-undef
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
