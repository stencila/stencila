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
  DocumentApp.super.apply(this, arguments);

  this.address = 'default';

  // Collaboration jam session is always null if a local
  // session and  `edit` or `comment` as necessary if remote
  this.jam = this.props.jam;
  if (this.props.local) {
    this.jam = null;
  } else if (!this.jam) {
    if (this.edit) this.jam = 'edit';
    else if (this.comment) this.jam = 'comment';
  }

  // Setup collaboration
  this.jamClient = new DocumentClient({
    httpUrl: 'http://localhost:5000/'
  });
  this.collabConn = new WebSocketConnection({
    wsUrl: 'ws://localhost:5000/'
  });
  this.collabClient = new CollabClient({
    connection: this.collabConn
  });

  // Bind to events
  this.handleActions({
    'reveal-toggle': this.toggleReveal,
    'comment-toggle': this.toggleComment,
    'edit-toggle': this.toggleEdit,
  });

}

DocumentApp.Prototype = function() {

  this.getInitialState = function() {
    // Initially, if in edit mode, then also turn on reveal mode
    // and comment mode (user can turn off these later if they want to)
    // See also `this.toggleEdit`
    var edit = this.props.edit;
    var reveal = this.props.reveal || edit;
    var comment = this.props.comment || edit;
    return {
      reveal: reveal,
      comment: comment,
      edit: edit,
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
      var session = null;
      var jam = null;
      if (this.jam) {
        jam = {
          name: this.jam,
          people : Object.keys(this.state.documentSession.collaborators).length + 1
        };
      }
      el.append(
        $$(VisualEditor, {
          // Document state
          session: session,
          jam: jam,
          reveal: this.state.reveal,
          comment: this.state.comment,
          edit: this.state.edit,
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
    if (!this.jam) {

      // Import the HTML provided from the page into a new document
      this.importHTML(this.props.html);

      // Create a new document session and add it to state to trigger
      // rerendering
      var documentSession = new DocumentSession(this.doc);
      this.extendState({
        documentSession: documentSession
      });

    } else {

      // Load initial Jam
      this.loadJam(this.jam);

    }
  };

  this.loadJam = function(jam) {
    this.jam = jam;

    this.extendState({
      documentSession: null
    });

    // Get the component jam from the `DocumentServer`...
    var jamId = this.address + ':' + this.jam;
    this.jamClient.getDocument('jam/' + jamId, function(err, componentJam) {

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
      this.extendState({
        documentSession: documentSession
      });

    }.bind(this));
  };

  this.requireJam = function(capability) {
    if (!this.props.local) {
      if (capability == 'edit' && this.jam != 'edit') {
        this.loadJam('edit');
      } else if (capability == 'comment' && !(this.jam == 'comment' || this.jam == 'edit')) {
        this.loadJam('comment');
      }
    }
  }

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

  /**
   * Toggle the `reveal` state
   */
  this.toggleReveal = function() {
    this.extendState({
      reveal: !this.state.reveal
    })
  }

  /**
   * Toggle the `comment` state
   */
  this.toggleComment = function() {
    this.extendState({
      comment: !this.state.comment
    });
    if (this.state.comment) this.requireJam('comment');
  }

  /**
   * Toggle the `edit` state. If edit mode is getting turned on
   * then reveal mode is also automatically turned on.
   */
  this.toggleEdit = function() {
    var edit = !this.state.edit;
    this.extendState({
      reveal: edit || this.state.reveal,
      comment: edit || this.state.comment,
      edit: edit
    });
    if (this.state.edit) this.requireJam('edit');
  }

};

Component.extend(DocumentApp);


module.exports = DocumentApp;
