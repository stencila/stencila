'use strict';

var Component = require('substance/ui/Component');
var DocumentSession = require('substance/model/DocumentSession');
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
var CodeEditor = require('./editors/code/CodeEditor');

/**
 * User application for a Stencila Document
 *
 * @class      DocumentApp (name)
 */
function DocumentApp () {
  DocumentApp.super.apply(this, arguments);

  // Bind to events
  this.handleActions({
    'view-toggle': this.toggleView,
    'reveal-toggle': this.toggleReveal,
    'comment-toggle': this.toggleComment,
    'edit-toggle': this.toggleEdit
  });
}

DocumentApp.Prototype = function () {
  this.getInitialState = function () {
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
  this.render = function ($$) {
    var el = $$('div').addClass('sc-document-app');

    if (this.state.documentSession) {
      var session = null;
      var copy = null;
      if (this.props.copy) {
        copy = {
          name: this.props.copy,
          people: Object.keys(this.state.documentSession.collaborators).length + 1
        };
      }
      var editorProps = {
        // Document state
        session: session,
        copy: copy,
        view: this.state.view,
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
        );
    }

    return el;
  };

  this.didMount = function () {
    var documentSession;

    if (this.props.format === 'html') {
      // Import the HTML provided from the page into a new document
      this.importHTML(this.props.data);

      // Create a new document session and add it to state to trigger
      // rerendering
      documentSession = new DocumentSession(this.doc);
      // For code compatability when using a `CollabSession`
      documentSession.config = {
        user: null,
        rights: null
      };
      this.extendState({
        documentSession: documentSession
      });
    } else {
      var user = this.props.data.user;
      var rights = this.props.data.rights;
      var collabUrl = this.props.data.collabUrl;
      var snapshot = this.props.data.snapshot;

      // Import the JSON
      this.importJSON(snapshot.data);

      // Create a new collaborative document session and add it to state
      // to trigger rerendering
      var collabConn = new WebSocketConnection({
        wsUrl: collabUrl
      });
      var collabClient = new CollabClient({
        connection: collabConn
      });
      documentSession = new CollabSession(this.doc, {
        documentId: snapshot.documentId,
        version: snapshot.version,
        collabClient: collabClient,
        user: user,
        rights: rights
      });
      this.extendState({
        documentSession: documentSession
      });
    }
  };

  this.importJSON = function (content) {
    this.doc = new DocumentModel();
    var jsonConverter = new DocumentJSONConverter();
    return jsonConverter.importDocument(this.doc, content);
  };

  this.exportJSON = function (content) {
    var jsonConverter = new DocumentJSONConverter();
    return jsonConverter.exportDocument(this.doc);
  };

  this.importHTML = function (content) {
    var htmlImporter = new DocumentHTMLImporter({
      configurator: configurator
    });
    this.doc = htmlImporter.importDocument(content);
  };

  this.exportHTML = function () {
    var htmlExporter = new DocumentHTMLExporter({
      configurator: configurator
    });
    return htmlExporter.exportDocument(this.doc);
  };

  /**
   * Toggle the view
   */
  this.toggleView = function (editor) {
    this.extendState({
      view: (this.state.view === 'visual') ? 'code' : 'visual'
    });
  };

  /**
   * Toggle the `reveal` state
   */
  this.toggleReveal = function () {
    this.extendState({
      reveal: !this.state.reveal
    });
  };

  /**
   * Toggle the `comment` state
   */
  this.toggleComment = function () {
    var comment = !this.state.comment;
    if (comment) {
      this.switchCopy('live');
    }
    this.extendState({
      comment: comment
    });
  };

  /**
   * Toggle the `edit` state. If edit mode is getting turned on
   * then reveal mode is also automatically turned on.
   */
  this.toggleEdit = function () {
    var edit = !this.state.edit;
    if (edit) {
      this.switchCopy('live');
    }
    this.extendState({
      reveal: edit || this.state.reveal,
      comment: edit || this.state.comment,
      edit: edit
    });
  };

  /**
   * Switch to a different copy (if necessary)
   *
   * @param      {string}  copy   The copy
   */
  this.switchCopy = function (copy) {
    if (this.props.copy !== copy) {
      this.extendState({
        documentSession: null
      });
      window.location = window.location + '@' + copy;
    }
  };
};

Component.extend(DocumentApp);

module.exports = DocumentApp;
