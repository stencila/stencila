'use strict';

var ToggleEditorStateCommand = require('../ToggleEditorStateCommand');

/**
 * Command for toggling edit mode of a Stencila Document
 * VisualEditor
 * 
 * In edit mode the content of the document can be
 * altered.
 *
 * @class      EditCommand (name)
 */
function EditCommand(params) {
  EditCommand.super.apply(this, arguments);
}

EditCommand.Prototype = function() {

  this.getStateName = function() {
    return 'edit';
  }

};

ToggleEditorStateCommand.extend(EditCommand);

module.exports = EditCommand;
