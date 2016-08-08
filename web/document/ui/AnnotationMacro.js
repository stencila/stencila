'use strict';

var Macro = require('./Macro');
var replaceText = require('substance/model/transform/replaceText');
var insertText = require('substance/model/transform/insertText');

function AnnotationMacro () {
}

AnnotationMacro.Prototype = function() {
  
  this.performAction = function(match, props, context) {
    var surface = context.surfaceManager.getSurface(props.selection.surfaceId);
    surface.transaction(function(tx, args) {

      var data = this.createNodeData(match);

      // Replace matched text
      var selection = tx.createSelection(props.path, match.index, match.index + match[0].length);
      var newText = replaceText(tx, {
        selection: selection,
        text: data.text
      });

      // Create annotation
      tx.create({
        type: data.type,
        path: newText.selection.path,
        startOffset: newText.selection.startOffset - newText.text.length,
        endOffset: newText.selection.endOffset
      });

      // Insert a space to end the annotation
      // CHECK Is there a better way to do this?
      // When you create a selection at end of `newText` it is still annotated
      tx.update(newText.selection.path, { insert: { offset: newText.selection.startOffset, value: ' ' } } );

      // Put selection just after annotation
      args.selection = tx.createSelection(newText.selection.path, newText.selection.endOffset + 1);

      return args;

    }.bind(this));
  };

};

Macro.extend(AnnotationMacro);

module.exports = AnnotationMacro;
