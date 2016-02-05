'use strict';

var isString = require('lodash/lang/isString');
var oo = require('substance/util/oo');
var uuid = require('substance/util/uuid');
var Component = require('substance/ui/Component');
var $$ = Component.$$;
var CellEditor = require('./CellEditor');

var Expression = require('./Expression');
var Constant = require('./Constant');

function CellComponent() {
  CellComponent.super.apply(this, arguments);
}

CellComponent.Prototype = function() {

  this.dispose = function() {
    this._disconnect();
  };

  this.render = function() {
    var node = this.props.node;
    var el = $$('td').addClass('se-cell');
    var componentRegistry = this.context.componentRegistry;
    var isEditing = this.isEditing();
    el.addClass(isEditing ? 'sm-edit' : 'sm-display');

    if (isEditing) {
      var content;
      if (isString(this.state.initialContent)) {
        content = this.state.initialContent;
      } else if (node) {
        content = node.content;
      } else {
        content = '';
      }
      el.append($$(CellEditor, {
        content: content,
        select: this.state.initialContent ? 'last' : 'all'
      }).ref('editor'));
    } else {
      el.on('dblclick', this.onDblClick);

      // Display node content
      if (node) {
        var CellContentClass;
        if (node.isConstant()) {
          CellContentClass = Constant;
        } else if (node.valueType) {
          CellContentClass = componentRegistry.get(node.valueType);
        }
        if (!CellContentClass) {
          CellContentClass = Expression;
        }
        var cellContentEl = $$(CellContentClass, {node: node}).ref('content');
        el.append(cellContentEl);

        var displayMode = node.displayMode || CellContentClass.static.displayModes[0];
        if (displayMode) {
          el.addClass('sm-'+displayMode);
        }
      }
    }
    return el;
  };

  this.didMount = function() {
    this._connect();
  };

  this.dispose = function() {
    this._disconnect();
  };

  this.willReceiveProps = function() {
    this._disconnect();
  };

  this.didReceiveProps = function() {
    this._connect();
  };

  this.getNode = function() {
    return this.props.node;
  };

  this.getDocument = function() {
    return this.context.doc;
  };

  this.getDocumentSession = function() {
    return this.context.documentSession;
  };

  /**
    There are 3 differnt display modes for cells

    clipped: uses minimal space
    expanded: displays all content available
    overlay: displays all content available
  */
  this.toggleDisplayMode = function() {
    var node = this.props.node;
    // empty cells do not have a node
    if (!node) return;

    var currentMode = node.displayMode;
    var content = this.refs.content;
    if (content) {
      var modes = content.constructor.static.displayModes || [];
      var idx = modes.indexOf(currentMode)+1;
      if (modes.length > 0) {
        idx = idx%modes.length;
      }
      var nextMode = modes[idx] || '';
      var docSession = this.getDocumentSession();
      docSession.transaction(function(tx) {
        tx.set([node.id, 'displayMode'], nextMode);
      }.bind(this));
    }
  };

  /**
    Ad-hoc creates a node when editing is enabled for an empty cell
  */
  this.enableEditing = function(initialContent) {
    this.extendState({
      edit: true,
      initialContent: initialContent
    });
  };

  this.commit = function() {
    var docSession = this.getDocumentSession();
    var doc = this.getDocument();
    var node = this.props.node;
    var newContent = this.getCellEditorContent() || '';
    if (!node) {
      var row = parseInt(this.attr('data-row'), 10);
      var col = parseInt(this.attr('data-col'), 10);
      var id = uuid();
      docSession.transaction(function(tx) {
        tx.create({
          type: "sheet-cell",
          id: id,
          row: row,
          col: col,
          content: newContent
        });
      });
      node = doc.get(id);
      this.extendProps({ node: node });
    } else if (newContent !== node.content) {
      docSession.transaction(function(tx) {
        tx.set([this.props.node.id, 'content'], newContent);
      }.bind(this));
      delete this.props.node.value;
    }
    this.extendState({ edit: false });
  };

  this.discard = function() {
    this.extendState({
      edit: false
    });
  };

  this.isEditing = function() {
    return this.state.edit;
  };

  this.getCellEditorContent = function() {
    if (this.refs.editor) {
      return this.refs.editor.getContent();
    }
  };

  this.onDblClick = function(e) {
    e.preventDefault();
    e.stopPropagation();
    this.enableEditing();
    this.send('activateCurrentCell');
  };

  this._connect = function() {
    var doc = this.getDocument();
    var node = this.props.node;
    if (node) {
      doc.getEventProxy('path').connect(this, [node.id, 'content'], this.rerender);
      doc.getEventProxy('path').connect(this, [node.id, 'displayMode'], this.rerender);
      node.connect(this, {
        'cell:changed': this._onCellChange // this.rerender
      });
    }
  };

  this._onCellChange = function() {
    this.rerender();
  };

  this._disconnect = function() {
    var doc = this.getDocument();
    if (this.props.node) {
      doc.getEventProxy('path').disconnect(this);
      this.props.node.disconnect(this);
    }
  };

};

oo.inherit(CellComponent, Component);

module.exports = CellComponent;
