'use strict';

var AbstractEditor = require('substance/ui/AbstractEditor');
var ScrollPane = require('substance/ui/ScrollPane');
var ContainerEditor = require('substance/ui/ContainerEditor');

var DocumentToolset = require('../../DocumentToolset');
var Overlayer = require('./Overlayer');
var MacroManager = require('../../ui/MacroManager');

/**
 * A editor for a Stencila Document
 *
 * @class      VisualEditor (name)
 */
function VisualEditor () {

  VisualEditor.super.apply(this, arguments);

  // Use custom MacroManager
  this.macroManager.context.documentSession.off(this.macroManager);
  delete this.macroManager;
  this.macroManager = new MacroManager(this.getMacroContext(), this.props.configurator.getMacros());

}

VisualEditor.Prototype = function () {

  /**
   * Render this editor
   */
  this.render = function ($$) {

    var configurator = this.props.configurator;

    var el = $$('div').addClass('sc-visual-editor');

    // Toggle classes to match properties
    ['reveal', 'edit'].forEach(function (item) {

      if (this.props[item]) el.addClass('sm-' + item);

    }.bind(this));

    // Document toolset (becuase of the way in which
    // tools and commands work, this has to go here, under an `AbstractEditor`,
    // instead of under the `DocumentApp`)
    el.append(
      $$(DocumentToolset, {
        copy: this.props.copy,
        view: this.props.view,
        reveal: this.props.reveal,
        comment: this.props.comment,
        edit: this.props.edit
      }).ref('documentToolset')
    );

    el.append(
      // A `ScrollPane` to manage overlays and other positioning
      $$(ScrollPane, {
        scrollbarType: 'native',
        scrollbarPosition: 'right',
        overlay: Overlayer
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
  this._documentSessionUpdated = function () {

    var commandStates = this.commandManager.getCommandStates();
    ['documentToolset'].forEach(function (name) {

      this.refs[name].extendProps({
        commandStates: commandStates
      });

    }.bind(this));

  };

};

AbstractEditor.extend(VisualEditor);

module.exports = VisualEditor;
