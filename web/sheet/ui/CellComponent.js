'use strict';

var oo = require('substance/util/oo');
var Component = require('substance/ui/Component');
var TextPropertyEditor = require('substance/ui/TextPropertyEditor');
var $$ = Component.$$;

function CellComponent() {
  CellComponent.super.apply(this, arguments);
}

CellComponent.Prototype = function() {

  this.render = function() {
    var node = this.props.node;
    var el = $$('td');

    var isEditing = this.isEditing();
    el.addClass(isEditing ? 'edit' : 'display');

    if (!isEditing) {
      el.on('dblclick', this.onDblClick);
      el.on('click', this.onClick);
    }

    if (node) {
      var isExpression = node.isExpression();
      el.addClass(isExpression ? 'expression' : 'text');
      if (isEditing) {
        var editor = $$(TextPropertyEditor, {
          name: node.id,
          path: [node.id, 'content'],
          commands: []
        }).ref('editor');
        el.append(editor);
      } else {
        if (isExpression) {
          el.text(node.value);
        } else {
          el.text(node.content);
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

  this.enableEditing = function() {
    this.setState({
      edit: true
    });
  };

  this.disableEditing = function() {
    this.setState({
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
    this.initializeSelection();
    this.send('activatedCell', this);
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
