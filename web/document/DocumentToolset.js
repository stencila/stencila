'use strict';

var Component = require('substance/ui/Component');
var Tool = require('substance/ui/Tool');


//var EditorTool = require('./tools/EditorTool');
var JamTool = require('./tools/JamTool');
var RefreshTool = require('./tools/RefreshTool');
var RevealTool = require('./tools/RevealTool');
var CommentTool = require('./tools/CommentTool');
var EditTool = require('./tools/EditTool');
var SaveTool = require('./tools/SaveTool');
var CommitTool = require('./tools/CommitTool');
var ForkTool = require('./tools/ForkTool');
var SettingsTool = require('./tools/SettingsTool');


function DocumentToolset() {
  DocumentToolset.super.apply(this, arguments);
}

DocumentToolset.Prototype = function() {

  this.render = function($$) {
    var el = $$('div')
      .addClass('sc-toolset sc-overall-toolset')
      .append(

        //$$(EditorTool, {
        //}).ref('editorTool'),

        $$(JamTool, {
          jam: this.props.jam
        }).ref('jamTool'),

        $$(RefreshTool, this._getCommandState('refresh'))
          .ref('refreshTool'),

        $$(RevealTool, {
          name: 'reveal',
          active: this.props.reveal
        }).ref('revealTool'),

        $$(CommentTool, {
          name: 'comment',
          active: this.props.comment
        }).ref('commentTool'),

        $$(EditTool, {
          name: 'edit',
          active: this.props.edit
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

Component.extend(DocumentToolset);

module.exports = DocumentToolset;
