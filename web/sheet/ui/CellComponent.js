'use strict';

var oo = require('substance/util/oo');
var uuid = require('substance/util/uuid');
var Component = require('substance/ui/Component');
var TextPropertyEditor = require('substance/ui/TextPropertyEditor');
var $$ = Component.$$;
var Sheet = require('../model/Sheet');

var TextContent = require('./TextComponent');
var ObjectComponent = require('./ObjectComponent');

function CellComponent() {
  CellComponent.super.apply(this, arguments);
}

CellComponent.Prototype = function() {

  this.render = function() {
    var cell = this.props.node;
    var componentRegistry = this.context.componentRegistry;
    var el = $$('td');
    var isEditing = this.isEditing();
    el.addClass(isEditing ? 'edit' : 'display');

    if (this.state.selected) {
      el.addClass('selected');
    }

    if (!isEditing) {
      el.on('dblclick', this.onDblClick);
      el.on('click', this.onClick);
    }

    if (cell) {
      el.addClass(cell.valueType);
      if (isEditing) {
        var editor = $$(TextPropertyEditor, {
          name: cell.id,
          path: [cell.id, 'content'],
          commands: []
        }).ref('editor');
        el.append(editor);
      } else {
        // Render Cell content
        var CellContentClass;
        if (cell.isPrimitive()) {
          CellContentClass = TextContent;
        } else if (cell.valueType) {
          CellContentClass = componentRegistry.get(cell.valueType);
        }
        if (!CellContentClass) {
          CellContentClass = ObjectComponent;
        }
        var cellContent = $$(CellContentClass, {node: cell, displayMode: this.state.displayMode});
        el.append(cellContent);
      }
    } else {
      el.addClass('empty');
    }
    return el;
  };

  // HACK: monkey patching the editor
  // Will come up with a dedicated Cell editor instead
  this.didRender = function() {
    var editor = this.refs.editor;
    if (editor) {
      editor._handleEnterKey = function(event) {
        this.disableEditing();
        this.send('commitCell', this, 'enter');
        event.stopPropagation();
        event.preventDefault();
      }.bind(this);
    }
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

  this.togglePreview = function() {
    this.extendState({
      preview: !this.state.preview
    });
  };

  /**
    There are 3 differnt display modes for cells

    clipped: uses minimal space
    expanded: displays all content available
    overlay: displays all content available
  */
  this.toggleDisplayMode = function() {
    var node = this.props.node;
    var currentMode = this.node.get('displayMode');
    var nextMode;
    var docSession = this.getDocumentSession();

    if (!currentMode || currentMode === 'overlay') {
      nextMode = 'clipped';
    } else if (currentMode === 'expanded') {
      nextMode = 'overlay';
    } else {
      nextMode = 'expanded';
    }

    docSession.transaction(function(tx) {
      tx.set([node.id, 'displayMode'], nextMode);
    }.bind(this));
  };

  /**
    Ad-hoc creates a node when editing is enabled for an empty cell
  */
  this.enableEditing = function() {
    if (!this.props.node) {
      var docSession = this.getDocumentSession();
      var doc = this.getDocument();
      var row = parseInt(this.attr('data-row'), 10);
      var col = parseInt(this.attr('data-col'), 10);

      var node = {
        type: "sheet-cell",
        id: uuid(),
        row: row,
        col: col,
        cid: Sheet.static.getCellId(row,col)
      };
      docSession.transaction(function(tx) {
        tx.create(node);
      });
      node = doc.get(node.id);
      this.extendProps({ node: node });
    }
    this.extendState({ edit: true });
    this.initializeSelection();
    this.send('activateCell', this);
  };

  this.disableEditing = function() {
    this.extendState({
      edit: false
    });
  };

  this.isEditing = function() {
    return this.state.edit;
  };

  /**
    Selects the full source text
  */
  this.initializeSelection = function() {
    var editor = this.refs.editor;
    if (editor) {
      editor.selectAll();
    }
  };

  this.onDblClick = function(e) {
    e.preventDefault();
    e.stopPropagation();
    this.enableEditing();
  };

  this.onClick = function(e) {
    if (!this.isEditing()) {
      e.preventDefault();
      e.stopPropagation();
      this.send('selectCell', this);
    }
  };

};

oo.inherit(CellComponent, Component);

module.exports = CellComponent;
