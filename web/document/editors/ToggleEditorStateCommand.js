'use strict';

var Command = require('substance/ui/Command');

/**
 * Command for toggling a state of a Stencila Document
 * editor
 * 
 * Most Substance commands update the document state,
 * this one updates the state of the editor.
 * So it involves a bit of hackery to deal with that.
 *
 * @class      ToggleEditorStateCommand (name)
 */
function ToggleEditorStateCommand() {
  ToggleEditorStateCommand.super.apply(this, arguments);

  // Because of the order in which `getCommandState`, 
  // and `editor.extendState` are called it is necessary for this 
  // command to maintain its own value of the state 
  this.active = null;
}

ToggleEditorStateCommand.Prototype = function() {

  /**
   * Get the name of the state variable which is
   * to be toggled. Override this in derived classes
   */
  this.getStateName = function(){
    throw new Error('This method is abstract.');
  }

  /**
   * Override of `Command.getCommandState`
   *
   * @param      {<type>}  props    The properties
   * @param      {<type>}  context  The context
   * @return     {Object}  The command state.
   */
  this.getCommandState = function(props, context) {
    // Initialise the active switch based on the
    // state of the editor
    if (this.active == null) {
      this.active = context.editor.state[this.getStateName()];
    }
    return {
      disabled: false,
      active: this.active
    };
  };

  /**
   * Override of `Command.execute`
   *
   * @param      {<type>}   props    The properties
   * @param      {<type>}   context  The context
   * @return     {boolean}  { description_of_the_return_value }
   */
  this.execute = function(props, context) {
    // Toggle active switch
    this.active = !this.active;
    // Since this command does not trigger a document update
    // we need to trigger an update to command states BEFORE
    // extending the state of, and thus rerending, the editor
    context.editor.commandManager.updateCommandStates();
    // Update the editor state
    var state = {};
    state[this.getStateName()] = this.active;
    context.editor.extendState(state);
    return true;
  }

};

Command.extend(ToggleEditorStateCommand);

module.exports = ToggleEditorStateCommand;
