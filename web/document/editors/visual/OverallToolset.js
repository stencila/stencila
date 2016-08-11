'use strict';

var Component = require('substance/ui/Component');
var Tool = require('substance/ui/Tool');

var RefreshTool = require('./RefreshTool');
var RevealTool = require('./RevealTool');
var CommentTool = require('./CommentTool');
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
    var toolRegistry = this.context.toolRegistry;

    var el = $$('div').addClass('sc-toolset sc-overall-toolset');

    el.append(
      $$(RefreshTool, this._getCommandState('refresh'))
        .ref('refreshTool'),
      $$(RevealTool, {
        name: 'reveal',
        active: this.parent.state.reveal
      }).ref('revealTool'),
      $$(CommentTool, {
        name: 'comment',
        active: this.parent.state.comment
      }).ref('commentTool'),
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

  /**
   * Convieience method to deal with necessary hack
   * to add command name to state for Substance `Tools` to render
   * icons
   */
  this._getCommandState = function(name){
      var state = this.context.commandManager.getCommandStates()[name];
      if (!state) throw new Error('Command {' + name + '} not found');
      state.name = name; // A necessary hack at time of writing
      return state;
  }

};

Component.extend(OverallToolset);

module.exports = OverallToolset;
