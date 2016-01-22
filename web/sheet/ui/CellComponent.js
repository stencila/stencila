'use strict';

var oo = require('substance/util/oo');
var uuid = require('substance/util/uuid');
var Component = require('substance/ui/Component');
var $$ = Component.$$;
var Sheet = require('../model/Sheet');
var CellEditor = require('./CellEditor');

var TextContent = require('./TextComponent');
var ObjectComponent = require('./ObjectComponent');

function CellComponent() {
  CellComponent.super.apply(this, arguments);

  // need to call this as willReceiveProps is only called when updating props
  this._connect();
}

CellComponent.Prototype = function() {

  this.render = function() {
    var cell = this.props.node;
    var componentRegistry = this.context.componentRegistry;
    var el = $$('td').addClass('se-cell');

    var isEditing = this.isEditing();
    el.addClass(isEditing ? 'edit' : 'display');

    if (!isEditing) {
      el.on('dblclick', this.onDblClick);
      // el.on('click', this.onClick);
    }

    if (cell) {
      // Mark as selected
      el.addClass(cell.valueType);
      if (isEditing) {
        el.append($$(CellEditor, {
          content: cell.content
        }).ref('editor'));
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
        var cellContent = $$(CellContentClass, {
          // HACK: having trouble with preservative rerendering
          // when Components are use with the same props
          // this hack forces a rerender
          hack: Date.now(),
          node: cell
        });
        el.append(cellContent);
      }
    } else {
      el.addClass('empty');
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

  this.getCellEditorContent = function() {
    if (this.refs.editor) {
      return this.refs.editor.getContent();
    }
  };

  this.onDblClick = function(e) {
    e.preventDefault();
    e.stopPropagation();
    this.enableEditing();
  };

  // this.onClick = function(e) {
  //   if (!this.isEditing()) {
  //     e.preventDefault();
  //     e.stopPropagation();
  //     this.send('selectCell', this);
  //   }
  // };

  this._connect = function() {
    var doc = this.getDocument();
    var node = this.props.node;
    if (node) {
      doc.getEventProxy('path').connect(this, [node.id, 'content'], this.rerender);
      doc.getEventProxy('path').connect(this, [node.id, 'displayMode'], this.rerender);
      node.connect(this, {
        'cell:changed': this.rerender
      });
    }
  };

  // this._onCellChange = function() {
  //   console.log('CELL CHANGED', this.props.node);
  //   this.rerender();
  // };

  this._disconnect = function() {
    var doc = this.getDocument();
    if (this.props.node) {
      doc.getEventProxy('path').disconnect(this);
      node.disconnect(this);
    }
  };

};

oo.inherit(CellComponent, Component);

module.exports = CellComponent;
