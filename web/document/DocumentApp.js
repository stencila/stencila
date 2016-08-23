'use strict';

var Component = require('substance/ui/Component');
var DocumentSession = require('substance/model/DocumentSession');
var CollabClient = require('substance/collab/CollabClient');
var CollabSession = require('substance/collab/CollabSession');
var WebSocketConnection = require('substance/collab/WebSocketConnection');
var request = require('substance/util/request');

var DocumentModel = require('./DocumentModel');
var DocumentJSONConverter = require('./DocumentJSONConverter');
var DocumentHTMLImporter = require('./DocumentHTMLImporter');
var DocumentHTMLExporter = require('./DocumentHTMLExporter');
    
// Instantiate a configurator
var DocumentConfigurator = require('./DocumentConfigurator');
var configurator = new DocumentConfigurator();

var VisualEditor = require('./editors/visual/VisualEditor');
var CodeEditor = require('./editors/code/CodeEditor');

/**
 * User application for a Stencila Document
 *
 * @class      DocumentApp (name)
 */
function DocumentApp() {
  DocumentApp.super.apply(this, arguments);

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
    var view = this.props.view;
    var edit = this.props.edit;
    var reveal = this.props.reveal || edit;
    var comment = this.props.comment || edit;
    return {
      view: view,
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

      var session = null;
      var clone = null;
      if (this.props.clone) {
        clone = {
          name: this.props.clone,
          people : Object.keys(this.state.documentSession.collaborators).length + 1
        };
      }
      var editorProps =  {
        // Document state
        session: session,
        clone: clone,
        reveal: this.state.reveal,
        comment: this.state.comment,
        edit: this.state.edit,
        // Other required props
        documentSession: this.state.documentSession,
        configurator: configurator
      };

      var view;
      if (this.state.view === 'visual') {
        view = $$(VisualEditor, editorProps).ref('visualEditor');
      } else {
        view = $$(CodeEditor, editorProps).ref('codeEditor');
      }
      el.append(view);

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
    if (this.props.format === 'html') {

      // Import the HTML provided from the page into a new document
      this.importHTML(this.props.data);

      // Create a new document session and add it to state to trigger
      // rerendering
      var documentSession = new DocumentSession(this.doc);
      this.extendState({
        documentSession: documentSession
      });

    } else {

      // ... import the JSON
      this.importJSON(this.props.data.data);

      // ... create a new collaborative document session and add it to state
      // to trigger rerendering
      var collabConn = new WebSocketConnection({
        wsUrl: this.props.data.collabUrl
      });
      var collabClient = new CollabClient({
        connection: collabConn
      });
      var documentSession = new CollabSession(this.doc, {
        documentId: this.props.data.documentId,
        version: this.props.data.version,
        collabClient: collabClient
      });
      this.extendState({
        documentSession: documentSession
      });

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

  /**
   * Change the editor
   */
  this.changeEditor = function(editor) {
    this.extendState({
      editor: editor
    })
  }

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
    var comment = !this.state.comment;
    if (comment) {
      this.switchClone('all');
    }
    this.extendState({
      comment: comment
    });
  }

  /**
   * Toggle the `edit` state. If edit mode is getting turned on
   * then reveal mode is also automatically turned on.
   */
  this.toggleEdit = function() {
    var edit = !this.state.edit;
    if (edit) {
      this.switchClone('all');
    }
    this.extendState({
      reveal: edit || this.state.reveal,
      comment: edit || this.state.comment,
      edit: edit
    });
  }

  /**
   * Switch to a different clone (if necessary)
   *
   * @param      {string}  clone   The clone
   */
  this.switchClone = function(clone) {
    if (this.props.clone !== clone) {
      this.extendState({
        documentSession: null
      });
      window.location = window.location + '@' + clone;
    }
  }

};

Component.extend(DocumentApp);


module.exports = DocumentApp;
