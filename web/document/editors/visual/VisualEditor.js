'use strict';

var AbstractEditor = require('substance/ui/AbstractEditor');
var ContainerEditor = require('substance/ui/ContainerEditor');

var Toolset = require('../Toolset');
var OverallToolset = require('./OverallToolset');
var AnnotationToolset = require('./AnnotationToolset');

/**
 * A editor for a Stencila Document
 *
 * @class      VisualEditor (name)
 */
function VisualEditor() {
  VisualEditor.super.apply(this, arguments);
}

VisualEditor.Prototype = function() {

  /**
  * Get the initial state of the editor
  *
  * @return     {Object}  The initial state.
  */
  this.getInitialState = function() {
    return {
      reveal: false,
      edit: this.props.rights=='write'
    };
  };

  /**
   * Render this editor
   */
  this.render = function($$) {
    var configurator = this.props.configurator;
    var toolRegistry = configurator.getToolRegistry();
    var commandStates = this.commandManager.getCommandStates();

    var el = $$('div').addClass('document-editor');

    // Toggle classes to match state and update
    // the extracted command states so relevant tools are
    // updated accordingly
    console.log(this.state);
    console.log(commandStates.edit);
    ['reveal', 'edit'].forEach(function(item) {
      var on = this.state[item];
      if (on) {
        el.addClass(item);
      } else {
        el.removeClass(item);
      }
    }.bind(this));

    // A Toolset for whole document commands
    el.append(
      $$(OverallToolset,{
        reveal: this.state.reveal,
        edit: this.state.edit,
        commandStates: commandStates
      }).ref('overallToolset')
    );

    if (this.state.edit) {

      // A Toolset to change the node type
      el.append(
        $$(Toolset, {
          toolList: ['switch-text-type'],
          toolRegistry: toolRegistry,
          commandStates: commandStates
        }).addClass('node-toolset')
          .ref('nodeToolset')
      );

      // A Toolset for annotation commands
      // This should only appear when there is a user text selection or when the cursor
      // is on an existing annoation
      el.append(
        $$(AnnotationToolset, {
          toolRegistry: toolRegistry,
          commandStates: commandStates
        }).ref('annotationToolset')
      );

    }
    
    // A ContainerEditor for the content of the document
    var content = $$(ContainerEditor, {
      containerId: 'content',
      disabled: !this.state.edit,
      commands: configurator.getSurfaceCommandNames(),
      textTypes: configurator.getTextTypes()
    }).ref('content');
    el.append(content);

    return el;
  };

  /**
   * Update editor when document session is updated.
   * 
   * This is an override of `AbstractEditor._documentSessionUpdated`
   * that instead of updating a single toolbar updates our multiple
   * toolsets.
   */
  this._documentSessionUpdated = function() {
    var commandStates = this.commandManager.getCommandStates();
    ['overallToolset', 'nodeToolset', 'annotationToolset'].forEach(function(name) {
      this.refs[name].extendProps({
        commandStates: commandStates
      });
    }.bind(this));

    var selection = this.documentSession.getSelection();
    var nodeId = selection.getNodeId();
    var el = document.querySelector('[data-id='+nodeId+']');
    var rect = el.getBoundingClientRect();
    this.refs.nodeToolset.extendProps({
      top: rect.top
    });
  };

};

AbstractEditor.extend(VisualEditor);


module.exports = VisualEditor;
