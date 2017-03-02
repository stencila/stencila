import { Component, forEach } from 'substance'

const MODES = {
  'cli': 'Clipped',
  'exp': 'Expanded',
  'ove': 'Overlay'
}

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
        //$$('span').addClass('se-label').append('Cell Mode: '),
        $$('span')
          .addClass('se-name')
          .append(this._getMode()),
        $$('span').addClass('se-dropdown-icon')
          .append(
            this.context.iconProvider.renderIcon($$, 'dropdown')
          )
      )
    el.append(toggleButton)

    if (this.state.expanded) {
      var availableModes = $$('div').addClass('se-available-display-modes')
      forEach(MODES, function(modeName, key) {
        var opt = $$('div').addClass('se-display-mode')
          .on('click', this._selectDisplayMode)
          .attr({
            'data-id': key
          }).append(modeName)

        availableModes.append(opt)
      }.bind(this))
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
    var mode = 'cli'
    if (this.state.cell) {
      mode = this.state.cell.displayMode
    }

    // We just 'Clipped' if mode is not found in availableNodes
    var displayMode = this.availableModes[mode] || 'Clipped'
    return displayMode
  }

  _toggleOptions() {
    if (!this.state.disabled) {
      this.extendState({
        expanded: !this.state.expanded
      })
    }
  }
}
