'use strict';

var ToggleEditorStateCommand = require('../ToggleEditorStateCommand');

/**
 * Command for toggling reveal mode of a Stencila Document
 * VisualEditor
 * 
 * In reveal mode, the directives in the document are shown.
 *
 * @class      RevealCommand (name)
 */
function RevealCommand() {
  RevealCommand.super.apply(this, arguments);
}

RevealCommand.Prototype = function() {

  this.getStateName = function() {
    return 'reveal';
  }

};

ToggleEditorStateCommand.extend(RevealCommand);

module.exports = RevealCommand;
