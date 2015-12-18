'use strict';

var oo = require('substance/util/oo');
var uuid = require('substance/util/uuid');
var Component = require('substance/ui/Component');
var TextPropertyEditor = require('substance/ui/TextPropertyEditor');
var $$ = Component.$$;

var Sheet = require('../model/Sheet');

function CellComponent() {
  CellComponent.super.apply(this, arguments);
}

CellComponent.Prototype = function() {

  this.render = function() {
    var node = this.props.node;
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

    if (node) {
      el.addClass(node.tipe);
      if (isEditing) {
        var editor = $$(TextPropertyEditor, {
          name: node.id,
          path: [node.id, 'expr'],
          commands: []
        }).ref('editor');
        el.append(editor);
      } else {
        var type = node.tipe;
        if (type=="integer" || type=="real" || type=="string"){
          el.text(node.value);
        } else if (type=="image-url"){
          el.append(
            $$('img')
              .attr('src', node.value)
          );
        } else {
          el
            .addClass('object')
            .append(
              $$('span')
                .text(node.tipe)
            )
            .append(
              $$('pre')
                .text(node.value)
            );
        }
      }
    } else {
      el.addClass('empty');
    }

    return el;
  };

  this.getNode = function() {
    return this.props.node;
  };

  this.getDocument = function() {
    return this.context.doc;
  };

  this.enableEditing = function() {
    if (!this.props.node) {
      var doc = this.getDocument();
      var row = new Number(this.attr('data-row'));
      var col = new Number(this.attr('data-col'));
      var node = {
        type: "sheet-cell",
        id: uuid(),
        row: row,
        col: col,
        cid: Sheet.static.getCellId(row,col)
      };
      doc.transaction(function(tx) {
        tx.create(node);
      });
      node = doc.get(node.id);
      this.extendProps({ node: node });
    }
    this.extendState({ edit: true });
    this.initializeSelection();
    this.send('activatedCell', this);
  };

  this.disableEditing = function() {
    this.extendState({
      edit: false
    });
  };

  this.isEditing = function() {
    return this.state.edit;
  };

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
      this.send('selectedCell', this);
    }
  };

};

oo.inherit(CellComponent, Component);

module.exports = CellComponent;
