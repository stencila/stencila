'use strict';

var Component = require('substance/ui/Component');
var Controller = require('substance/ui/Controller');
var SheetComponent = require('./SheetComponent');
var $$ = Component.$$;

function SheetEditor() {
  SheetEditor.super.apply(this, arguments);

  this.handleActions({
    'selectedCell': this.onSelectedCell,
    'activatedCell': this.onActivatedCell,
  });
}

SheetEditor.Prototype = function() {

  this.didMount = function() {
    // HACK: to override the hacky parent implementation
  };

  this.render = function() {
    return $$('div').addClass('sheet-editor').ref('sheet')
      .append($$(SheetComponent, { doc: this.props.doc }));
  };

  this.onSelectedCell = function(cell) {
    if (this.activeCell && this.activeCell !== cell) {
      this.activeCell.disableEditing();
    }
    var node = cell.getNode();
    if (node && node.isExpression()) {
      console.log('Show expression bar.');
    }
  };

  this.onActivatedCell = function(cell) {
    if (this.activeCell && this.activeCell !== cell) {
      this.activeCell.disableEditing();
    }
    this.activeCell = cell;
  };

};

Controller.extend(SheetEditor);

module.exports = SheetEditor;
