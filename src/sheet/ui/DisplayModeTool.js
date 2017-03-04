import {Component} from 'substance'
import Cell from '../model/Cell'

export default
class DisplayModeTool extends Component {

  getInitialState() {
    return {
      disabled: true,
      expanded: false,
      cell: null, // focussed cell
    }
  }

  render($$) {
    var el = $$('div')
      .addClass('se-tool se-display-mode-tool')

    if (this.state.disabled) {
      el.addClass('sm-disabled')
      // We just hide the tool if disabled
      return el
    }
    if (this.state.expanded) {
      el.addClass('sm-expanded')
    }

    var toggleButton = $$('button')
      .on('click', this._toggleOptions)
      .append(
        $$('span')
          .addClass('se-name')
          .append(this.getLabel(this._getMode())),
        $$('span').addClass('se-dropdown-icon')
          .append(
            this.context.iconProvider.renderIcon($$, 'dropdown')
          )
      )
    el.append(toggleButton)

    if (this.state.expanded) {
      var availableModes = $$('div').addClass('se-available-display-modes')
      Cell.DISPLAY_MODES.forEach((modeName, key) => {
        var opt = $$('div').addClass('se-display-mode')
          .on('click', this._selectDisplayMode)
          .attr({
            'data-id': key
          }).append(this.getLabel(modeName))
        availableModes.append(opt)
      })
      el.append(availableModes)
    }
    return el
  }


  _selectDisplayMode(e) {
    var selectedMode = e.currentTarget.dataset.id
    var editorSession = this.context.editorSession
    var cell = this.state.cell
    editorSession.transaction(function(tx) {
      tx.set([cell.id, 'displayMode'], selectedMode)
    })
    this.extendState({
      expanded: false
    })
  }

  _getMode() {
    return this.state.cell ? this.state.cell.displayMode : 'cli'
  }

  _toggleOptions() {
    if (!this.state.disabled) {
      this.extendState({
        expanded: !this.state.expanded
      })
    }
  }
}
