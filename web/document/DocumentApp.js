'use strict';

var Component = require('substance/ui/Component');
var DocumentSession = require('substance/model/DocumentSession');

// Instantiate a configurator
var DocumentConfigurator = require('./DocumentConfigurator');
var configurator = new DocumentConfigurator();

// Instantiate an importer
var DocumentHTMLImporter = require('./DocumentHTMLImporter');
var importer = new DocumentHTMLImporter({
  configurator: configurator
});

// Instantiate an exporter
var DocumentHTMLExporter = require('./DocumentHTMLExporter');
var exporter = new DocumentHTMLExporter({
  configurator: configurator
});

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
    var doc = importer.importDocument(this.props.html);
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
    var el = $$('div').addClass('document-app');

    // Render the visual WYSIWYG editor
    el.append(
      $$(VisualEditor, {
        // Props of document that afftect editor
        rights: this.state.doc.rights,
        // Other required props
        documentSession: this.state.documentSession,
        configurator: configurator
      }).ref('editor')
    );

    return el;
  };

};

Component.extend(DocumentApp);


module.exports = DocumentApp;
