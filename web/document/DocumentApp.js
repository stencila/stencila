'use strict';

var Component = require('substance/ui/Component');
var DocumentSession = require('substance/model/DocumentSession');
var CollabClient = require('substance/collab/CollabClient');
var CollabSession = require('substance/collab/CollabSession');
var WebSocketConnection = require('substance/collab/WebSocketConnection');

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

  this.doc = DocumentModel.import(this.props.html);

  if (this.props.collab) {
    this.collabConn = new WebSocketConnection({
      wsUrl: 'ws://localhost:5000'
    });

    this.collabClient = new CollabClient({
      connection: this.collabConn
    });

    this.documentSession = new CollabSession(this.doc, {
      documentId: 'default',
      version: 1,
      collabClient: this.collabClient
    });
  } else {
    this.documentSession = new DocumentSession(this.doc);
  }

}

DocumentApp.Prototype = function() {

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
        collab: this.props.collab,
        // Props of document that affect editor
        rights: this.doc.rights,
        // Other required props
        documentSession: this.documentSession,
        configurator: configurator
      }).ref('visualEditor')
    );

    return el;
  };

};

Component.extend(DocumentApp);


module.exports = DocumentApp;
