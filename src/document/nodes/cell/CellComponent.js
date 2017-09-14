import CodeEditorComponent from '../../ui/CodeEditorComponent'
import CellValueComponent from './CellValueComponent'
import MiniLangEditor from './MiniLangEditor'
import CellErrorDisplay from './CellErrorDisplay'
import CodeblockComponent from '../codeblock/CodeblockComponent'

class CellComponent extends CodeblockComponent {

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
    const cell = this.props.node
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

    // TODO: bring back, as soon as we have an API
    // if (cell.hasErrors()) {
    //   toggleCell.addClass('sm-has-errors')
    // }
    el.append(toggleCell)

    if (this.state.showCode) {
      let cellEditorContainer = $$('div').addClass('se-cell-editor-container')
      cellEditorContainer.append(
        $$('div').addClass('se-expression').append(
          'TODO: bring back MiniLangEditor'
          // $$(MiniLangEditor, {
          //   path: [cell.id, 'expression'],
          //   excludedCommands: this._getBlackListedCommands(),
          //   expression: cell.getExpressionNode()
          // }).ref('expressionEditor')
          //   .on('escape', this._onEscapeFromCodeEditor)
        )
      )

      // if (cell.isExternal()) {
      //   cellEditorContainer.append(
      //     $$(CodeEditorComponent, {
      //       path: [cell.id, 'sourceCode'],
      //       language: cell.context
      //     }).ref('sourceCodeEditor')
      //       .on('escape', this._onEscapeFromCodeEditor)
      //   )
      // }

      el.append(cellEditorContainer)
      el.append(
        // TODO: bring back cell error display
        // $$(CellErrorDisplay, {cell})
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
    let isDefinition = this._isDefinition()
    if (!isDefinition) {
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
    let isDefinition = this._isDefinition()
    // No toggling allowed if cell is not a definition
    if (!isDefinition) return
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

  _toggleMenu() {
    this.extendState({
      showMenu: !this.state.showMenu
    })
  }

  _isDefinition() {
    // TODO: instead do return this.props.node.isDefinition()
    return false
  }

  _onExecute() {
    this.props.node.recompute()
  }

}

CellComponent.noBlocker = true

export default CellComponent
