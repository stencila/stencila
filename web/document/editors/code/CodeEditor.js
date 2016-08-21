'use strict';

var AbstractEditor = require('substance/ui/AbstractEditor');
var ScrollPane = require('substance/ui/ScrollPane');
var ContainerEditor = require('substance/ui/ContainerEditor');

var each = require('lodash/collection/each');

var DocumentConfigurator = require('../../DocumentConfigurator');
var DocumentToolset = require('../../DocumentToolset');
var MacroManager = require('../../ui/MacroManager');

/**
 * A editor for a Stencila Document
 *
 * @class      CodeEditor (name)
 */
function CodeEditor() {
  CodeEditor.super.apply(this, arguments);

  // Use custom MacroManager
  this.macroManager.context.documentSession.off(this.macroManager);
  delete this.macroManager;
  this.macroManager = new MacroManager(this.getMacroContext(), this.props.configurator.getMacros());
}

CodeEditor.Prototype = function() {

  /**
   * Render this editor
   */
  this.render = function($$) {
    var configurator = this.props.configurator;

    var el = $$('div').addClass('sc-code-editor');

    // Document toolset (becuase of the way in which
    // tools and commands work, this has to go here, under an `AbstractEditor`,
    // instead of under the `DocumentApp`)
    el.append(
      $$(DocumentToolset,{
        jam: this.props.jam,
        reveal: this.props.reveal,
        comment: this.props.comment,
        edit: this.props.edit
      }).ref('overallToolset')
    );

    // Change the `ComponentRegistry` used by the `ContainerEditor`
    // to decide which components to render for each mode type by
    // replacing the "defult" component type with the component type
    // for this code editor's language (currently just Markdown)
    var componentRegistry = configurator.getComponentRegistry();
    each(Object.keys(configurator.config.nodes), function(nodeType){
      var component = componentRegistry.get(nodeType + '-markdown');
      if (component) componentRegistry.add(nodeType, component);
    });
    this.componentRegistry = componentRegistry;

    el.append(
      // A `ScrollPane` to manage overlays and other positioning
      $$(ScrollPane, {
        scrollbarType: 'native',
        scrollbarPosition: 'right'
      })
        .ref('scrollPane')
        .append(
          // A  ContainerEditor  for the content of the document
          $$(ContainerEditor, {
            containerId: 'content',
            disabled: !this.props.edit,
            commands: configurator.getSurfaceCommandNames(),
            textTypes: configurator.getTextTypes()
          }).ref('containerEditor')
        )
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
    ['overallToolset'].forEach(function(name) {
      this.refs[name].extendProps({
        commandStates: commandStates
      });
    }.bind(this));
  };

};

AbstractEditor.extend(CodeEditor);


module.exports = CodeEditor;
