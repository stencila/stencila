'use strict';

var Component = require('substance/ui/Component');
var $$ = Component.$$;
var Icon = require('substance/ui/FontAwesomeIcon');
var each = require('lodash/collection/each');

function DisplayModeTool() {
  Component.apply(this, arguments);
}

DisplayModeTool.Prototype = function() {

  this.availableModes = {
    'cli': 'Clipped',
    'exp': 'Expanded',
    'ove': 'Overlay'
  };

  this.getInitialState = function() {
    return {
      disabled: true,
      expanded: false,
      cell: null, // focussed cell
    };
  };

  this._selectDisplayMode = function(e) {
    var selectedMode = e.currentTarget.dataset.id;
    var docSession = this.context.documentSession;
    var cell = this.state.cell;
    docSession.transaction(function(tx) {
      tx.set([cell.id, 'displayMode'], selectedMode);
    }.bind(this));
    // Rerender!
    this.extendState({
      expanded: false
    });
  };

  this._getMode = function() {
    var mode = 'cli';
    if (this.state.cell) {
      mode = this.state.cell.displayMode;
    }

    // We just 'Clipped' if mode is not found in availableNodes
    var displayMode = this.availableModes[mode] || 'Clipped';
    return displayMode;
  };

  this._toggleOptions = function() {
    if (!this.state.disabled) {
      this.extendState({
        expanded: !this.state.expanded
      });      
    }
  };

  this.render = function() {
    console.log('DisplayModeTool.render');

    var el = $$('div')
      .addClass('se-tool se-display-mode-tool');

    if (this.state.disabled) {
      el.addClass('sm-disabled');
      // We just hide the tool if disabled
      return el;
    }
    if (this.state.expanded) {
      el.addClass('sm-expanded');
    }

    var toggleButton = $$('button')
      .on('click', this._toggleOptions)
      .append(
        //$$('span').addClass('se-label').append('Cell Mode: '),
        $$('span')
          .addClass('se-name')
          .append(this._getMode()),
        $$('span').addClass('se-dropdown-icon')
          .append($$(Icon, {icon: 'fa-caret-down'}))
      );
    el.append(toggleButton);

    if (this.state.expanded) {
      var availableModes = $$('div').addClass('se-available-display-modes');
      each(this.availableModes, function(modeName, key) {
        var opt = $$('div').addClass('se-display-mode')
          .on('click', this._selectDisplayMode)
          .attr({
            'data-id': key
          }).append(modeName);

        availableModes.append(opt);
      }.bind(this));
      el.append(availableModes);
    }
    return el;
  };
};

Component.extend(DisplayModeTool);

module.exports = DisplayModeTool;
