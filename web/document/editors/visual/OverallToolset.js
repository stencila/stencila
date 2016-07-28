'use strict';

var Component = require('substance/ui/Component');
var Tool = require('substance/ui/Tool');

var Toolset = require('../Toolset');
var RevealTool = require('./RevealTool');
var RefreshTool = require('./RefreshTool');
var EditTool = require('./EditTool');
var SaveTool = require('./SaveTool');
var CommitTool = require('./CommitTool');
var ForkTool = require('./ForkTool');
var SettingsTool = require('./SettingsTool');


function OverallToolset() {
  Component.apply(this, arguments);
}

OverallToolset.Prototype = function() {

  this.render = function($$) {

    var el = $$('div').addClass('sc-toolset sc-overall-toolset');

    el.append(
      $$(RevealTool, {
        name: 'reveal',
        active: this.parent.state.reveal
      }).ref('revealTool'),
      $$(RefreshTool, this._getCommandState('refresh'))
        .ref('refreshTool'),
      $$(EditTool, {
        name: 'edit',
        active: this.parent.state.edit
      }).ref('editTool')
    );

    var editGroup = $$('div')
      .addClass('se-group se-edit-group')
      .ref('editGroup')
      .append(
        $$(Tool, this._getCommandState('undo')),
        $$(Tool, this._getCommandState('redo')),
        $$(SaveTool, this._getCommandState('save')),
        $$(CommitTool, this._getCommandState('commit'))
      );
    if (this.props.edit) {
      editGroup.addClass('sm-enabled');
    }
    el.append(editGroup);

    el.append(
      $$(ForkTool, this._getCommandState('fork'))
        .ref('forkTool'),
      $$(SettingsTool, this._getCommandState('settings'))
        .ref('settingsTool')
    );

    return el;
  };

};

Toolset.extend(OverallToolset);

module.exports = OverallToolset;
