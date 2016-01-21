'use strict';

var Component = require('substance/ui/Component');
var $$ = Component.$$;

/*
  The CellEditor is different to a regular TextPropertyEditor
  in the regard that it doesn't update the document during editing,
  only at the end.
*/
function CellEditor() {
  CellEditor.super.apply(this, arguments);
}

CellEditor.Prototype = function() {

  this.render = function() {
    var el = $$('textarea')
      .text(this.props.content)
      .on('keydown', this.onKeydown)
      .on('keypress', this.onKeypress);
    return el;
  };

  this.didMount = function() {
    this.el.selectionStart = 0;
    this.el.selectionEnd = this.props.content.length;
  };

  this.getContent = function() {
    return this.el.textContent;
  };

  this.onKeydown = function(event) {
    console.log('CellEditor.onKeydown()', 'keyCode=', event.keyCode, event);
    var handled = false;
    // ENTER
    if (event.keyCode === 13) {
      if (!event.ctrlKey && !event.shiftKey) {
        console.log('TODO: commit cell change.');
        this.send('commitCellChange', this.el.textContent, 'enter');
        handled = true;
      }
    } else if (event.keyCode === 27) {
      console.log('TODO: discard cell change.');
      this.send('discardCellChange');
      handled = true;
    }
    if (handled) {
      event.stopPropagation();
      event.preventDefault();
    }
  };

  this.onKeypress = function(event) {
    console.log('CellEditor.onKeypress()', 'keyCode=', event.keyCode);
    var handled = false;
    if (handled) {
      event.stopPropagation();
      event.preventDefault();
    }
  };

};

Component.extend(CellEditor);

module.exports = CellEditor;
