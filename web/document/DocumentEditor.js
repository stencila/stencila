'use strict';

var AbstractEditor = require('substance/ui/AbstractEditor');
var ContainerEditor = require('substance/ui/ContainerEditor');

var Toolset = require('./ui/Toolset');

/**
 * A editor for a Stencila Document
 *
 * @class      DocumentEditor (name)
 */
function DocumentEditor() {
  DocumentEditor.super.apply(this, arguments);
}

DocumentEditor.Prototype = function() {

  /**
   * Render this editor
   */
  this.render = function($$) {
    var configurator = this.props.configurator;
    var toolRegistry = configurator.getToolRegistry();
    var commandStates = this.commandManager.getCommandStates();

    var el = $$('div').addClass('document-editor');

    // A Toolset for whole document commands
    el.append(
      $$(Toolset, {
        toolRegistry: toolRegistry,
        toolList: ['undo', 'redo'],
        commandStates: commandStates
      }).addClass('document-toolset').ref('document_toolset')
    );

    // A Toolset for annotation commands
    el.append(
      $$(Toolset, {
        toolRegistry: toolRegistry,
        toolList: ['emphasis', 'strong', 'subscript', 'superscript', 'code', 'link'],
        commandStates: commandStates
      }).addClass('text-toolset').ref('text_toolset')
    );
    
    // A ContainerEditor for the content of the document
    el.append(
      $$(ContainerEditor, {
        containerId: 'content',
        disabled: this.props.mode != 'write',
        commands: configurator.getSurfaceCommandNames(),
        textTypes: configurator.getTextTypes()
      }).ref('content')
    );

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
    ['document_toolset', 'text_toolset'].forEach(function(name) {
      this.refs[name].extendProps({
        commandStates: commandStates
      });
    }.bind(this));
  };

};

AbstractEditor.extend(DocumentEditor);


module.exports = DocumentEditor;
