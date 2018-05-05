import { Component, NodeComponent, FontAwesomeIcon, isEqual } from 'substance'
import ValueComponent from '../shared/ValueComponent'
import CodeEditor from '../shared/CodeEditor'
import { getCellState, getErrorMessage } from '../shared/cellHelpers'
import { toString as stateToString, OK, BROKEN, FAILED } from '../engine/CellStates'
import NodeMenu from './NodeMenu'

const LANG_LABELS = {
  'mini': 'Mini',
  'js': 'JS',
  'node': 'Node',
  'sql': 'SQL',
  'py': 'Py',
  'r': 'R',
}

const SHOW_ERROR_DELAY = 500

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
      let valueDisplay = $$(ValueDisplay, {
        status: cellState.status,
        value: cellState.value,
        errors: cellState.errors,
        showOutput: this._showOutput(),
      }).ref('valueDisplay')
      el.append(valueDisplay)
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

class ValueDisplay extends Component {

  shouldRerender(newProps) {
    return (
      (newProps.showOutput !== this.props.showOutput) ||
      (newProps.status !== this.props.status) ||
      (newProps.value !== this.props.value) ||
      (!isEqual(newProps.errors, this.props.errors))
    )
  }

  willReceiveProps(newProps) {
    let newStatus = newProps.status
    if (newStatus === OK) {
      this._cachedValue = newProps.value
      // this._cachedError = null
    } else if (newStatus === BROKEN || newStatus === FAILED) {
      this._cachedError = newProps.errors[0]
      // this._cachedValue = null
    }
  }

  didUpdate() {
    const errors = this.props.errors
    if (errors && errors.length > 0) {
      let token = this._token
      setTimeout(() => {
        // if this is still the same update
        if (token === this._token) {
          if (this.refs.cachedValue) {
            this.refs.cachedValue.css('display', 'none')
          }
          if (this.refs.error) {
            this.refs.error.css('display', 'block')
          }
        }
      }, SHOW_ERROR_DELAY)
    }
  }

  render($$) {
    const status = this.props.status
    const value = this.props.value
    const showOutput = this.props.showOutput
    const errors = this.props.errors
    let el = $$('div')
    // whenever there are errors we will renew the token
    // so that an already triggered updater can be canceled
    this._token = Math.random()
    if(status === BROKEN || status === FAILED) {
      // rendering the last value as hidden to achieve a bit more resilient spacing
      if (this._cachedValue && showOutput) {
        el.append(
          $$(ValueComponent, this._cachedValue).ref('cachedValue').css('visibility', 'hidden')
        )
      } else if (this._cachedError) {
        el.append(
          $$('div').addClass('se-error').append(
            getErrorMessage(this._cachedError)
          ).ref('cachedValue').css('visibility', 'hidden')
        )
      }
      // ... and the error is not shown at first, but didUpdate() will show it after some delay
      // this way the error is a bit delayed, potentially becoming superseded by a new update in the meantime
      el.append(
        $$('div').addClass('se-error').append(
          getErrorMessage(errors[0])
        ).ref('error').css('display', 'none')
      )
    } else if (showOutput) {
      if (value && status === OK) {
        el.append(
          $$(ValueComponent, value).ref('value')
        )
      }
      // to have a less jumpy experience, we show the last valid value grey'd out
      else if (this._cachedValue) {
        el.append(
          $$(ValueComponent, this._cachedValue).ref('value').addClass('sm-pending')
        )
      }
    }
    return el
  }
}

CellComponent.noBlocker = true
