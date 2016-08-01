'use strict';

var Component = require('substance/ui/Component');

var code = require('../../shared/code');

/**
 * A `Component` for editing a node's `source` code
 * attribute.
 * 
 * This is based heavily on https://github.com/substance/examples/blob/v1.0.0-beta.4/code-editor/script/ScriptEditor.js.
 * See there for extra notes.
 *
 * @class      CodeEditorComponent (name)
 */
function CodeEditorComponent() {
  CodeEditorComponent.super.apply(this, arguments);

  this.editor = null;
}

CodeEditorComponent.Prototype = function() {

  this.render = function($$) {
    var node = this.props.node;
    var el = $$('div')
      .addClass('sc-code-editor')
      .append(
        $$('pre')
          .ref('editor')
          .append(node.source)
      );
    return el;
  };

  this.didMount = function() {
    var node = this.props.node;
    // Attach ACE editor (allows for asynchronous loading of ACE)
    code.attachAceEditor(
      this.refs.editor.getNativeElement(),
      node.language,
      null,
      function(editor) {
        // When editor has been created...
        editor.on('blur', this._onEditorBlur.bind(this));
        this.editor = editor;
      }.bind(this)
    );
    node.on('source:changed', this._onContentChanged, this);
  };

  this.shouldRerender = function() {
    // Don't rerender as that would destroy editor
    return false;
  };

  this.dispose = function() {
    this.props.node.off(this);
    this.editor.destroy();
  };

  /**
   * When editor loses focus (blur) update
   * the node's source
   */
  this._onEditorBlur = function() {
    var editor = this.editor;
    var nodeId = this.props.node.id;
    var source = editor.getValue();
    if (source !== this.props.node.source) {
      this.context.surface.transaction(function(tx) {
        tx.set([nodeId, 'source'], source);
      }, { source: this, skipSelection: true });
    }
  };

  /**
   * When the node's source changes, update the 
   * editor (if this wasn't the source of the update)
   */
  this._onContentChanged = function(change, info) {
    if (info.source !== this && this.editor) {
      this.editor.setValue(this.props.node.source, -1);
    }
  }

};

Component.extend(CodeEditorComponent);

CodeEditorComponent.fullWidth = true;

module.exports = CodeEditorComponent;
