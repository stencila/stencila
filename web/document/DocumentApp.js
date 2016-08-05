'use strict';

var Component = require('substance/ui/Component');
var DocumentSession = require('substance/model/DocumentSession');

var DocumentModel = require('./DocumentModel');

// Instantiate a configurator
var DocumentConfigurator = require('./DocumentConfigurator');
var configurator = new DocumentConfigurator();

var VisualEditor = require('./editors/visual/VisualEditor');

/**
 * User application for a Stencila Document
 *
 * @class      DocumentApp (name)
 */
function DocumentApp() {
  Component.apply(this, arguments);
}

DocumentApp.Prototype = function() {

  /**
  * Get the initial state of the application
  *
  * @return     {Object}  The initial state.
  */
  this.getInitialState = function() {
    var doc = DocumentModel.import(this.props.html);
    var documentSession = new DocumentSession(doc);
    return {
      doc: doc,
      documentSession: documentSession
    };
  };

  /**
  * Render the application
  *
  * @param      {Function}  $$ Function for creating virtual nodes
  * @return     {VirtualNode}  Virtual node to be added to the DOM 
  */
  this.render = function($$) {
    var el = $$('div').addClass('sc-document-app');

    // Render the visual WYSIWYG editor
    el.append(
      $$(VisualEditor, {
        // Parameters of the app
        reveal: this.props.reveal,
        edit: this.props.edit,
        // Props of document that affect editor
        rights: this.state.doc.rights,
        // Other required props
        documentSession: this.state.documentSession,
        configurator: configurator
      }).ref('visualEditor')
    );

    return el;
  };

};

Component.extend(DocumentApp);


module.exports = DocumentApp;
