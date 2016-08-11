'use strict';

var Component = require('substance/ui/Component');
var DocumentSession = require('substance/model/DocumentSession');
var DocumentClient = require('substance/collab/DocumentClient');
var CollabClient = require('substance/collab/CollabClient');
var CollabSession = require('substance/collab/CollabSession');
var WebSocketConnection = require('substance/collab/WebSocketConnection');

var DocumentModel = require('./DocumentModel');
var DocumentJSONConverter = require('./DocumentJSONConverter');
var DocumentHTMLImporter = require('./DocumentHTMLImporter');
var DocumentHTMLExporter = require('./DocumentHTMLExporter');
    
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

  this.address = 'default';

  // Override capability settings based on rights
  // TODO
  this.reveal = this.props.reveal;
  this.comment = this.props.comment;
  this.edit = this.props.edit;

  // Collaboration setting `off` unless remote and comment or edit 
  // capabilities (in which case as set by caller)
  this.collab = false;
  if (!this.props.local && (this.comment || this.edit)) {
    this.collab = this.props.collab;
  }

  if (this.collab) {

    this.documentClient = new DocumentClient({
      httpUrl: 'http://localhost:5000/'
    });

    this.collabConn = new WebSocketConnection({
      wsUrl: 'ws://localhost:5000/'
    });

    this.collabClient = new CollabClient({
      connection: this.collabConn
    });

  }

}

DocumentApp.Prototype = function() {

  this.getInitialState = function() {
    return {
      documentSession: null,
      message: null
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

    if (this.state.documentSession) {
      // Render the visual WYSIWYG editor
      el.append(
        $$(VisualEditor, {
          // Capability settings
          reveal: this.reveal,
          comment: this.comment,
          edit: this.edit,
          collab: this.collab,
          // Other required props
          documentSession: this.state.documentSession,
          configurator: configurator
        }).ref('visualEditor')
      );
    } else {
      el
        .addClass('sm-loading')
        .append(
          $$('i')
            .addClass('fa fa-spinner fa-pulse fa-fw')
        );
    }

    if (this.state.message) {
      el
        .addClass('sm-message')
        .append(
          $$('p')
            .addClass('se-message')
            .text(this.state.message.string)
        )
    }

    return el;
  };

  this.didMount = function() {
    if (!this.collab) {

      // Import the HTML provided from the page into a new document
      this.importHTML(this.props.html);

      // Create a new document session and add it to state to trigger
      // rerendering
      var documentSession = new DocumentSession(this.doc);
      this.setState({
        documentSession: documentSession
      });

    } else {

      // Get the component jam from the `DocumentServer`...
      var jamId = this.address + '?jam=comment';
      this.documentClient.getDocument('jam/' + jamId, function(err, componentJam) {

        // ... display any errors
        if (err) {
          console.error(err);
          return this.extendState({
            message: {
              type: 'error',
              string: 'Unable to get document: ' + err
            }
          });
        }

        // ... import the JSON
        this.importJSON(componentJam.data);

        // ... create a new collaborative document session and add it to state
        // to trigger rerendering
        var documentSession = new CollabSession(this.doc, {
          documentId: componentJam.documentId,
          version: componentJam.version,
          collabClient: this.collabClient
        });
        this.setState({
          documentSession: documentSession
        });

      }.bind(this));

    }
  };

  this.importJSON = function(content) {
    this.doc = new DocumentModel();
    var jsonConverter = new DocumentJSONConverter();
    return jsonConverter.importDocument(this.doc, content);
  };

  this.exportJSON = function(content) {
    var jsonConverter = new DocumentJSONConverter();
    return jsonConverter.exportDocument(this.doc);
  };

  this.importHTML = function(content) {
    var htmlImporter = new DocumentHTMLImporter({
      configurator: configurator
    });
    this.doc = htmlImporter.importDocument(content);
  };

  this.exportHTML = function() {
    var htmlExporter = new DocumentHTMLExporter({
      configurator: configurator
    });
    return htmlExporter.exportDocument(this.doc);
  };

};

Component.extend(DocumentApp);


module.exports = DocumentApp;
