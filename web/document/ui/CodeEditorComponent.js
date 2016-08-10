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

  // In `this._onCodeChanged` and `this._onLanguageChanged`, these custom props
  // are not on `this.props` for some reason. So, "store" them here.
  this.codeProperty = this.props.codeProperty;
  this.languageProperty = this.props.languageProperty;
}

CodeEditorComponent.Prototype = function() {

  this.render = function($$) {
    var node = this.props.node;
    return $$('div')
      .addClass('sc-code-editor')
      .append(
        $$('div')
          .ref('editor')
          .text(node[this.props.codeProperty])
      );
  };

  this.didMount = function() {
    var node = this.props.node;

    // Resolve the language for the code
    var language;
    if (this.props.languageProperty) {
      language = node[this.props.languageProperty];
    } else {
      language = this.props.language;
    }

    // Attach ACE editor (allows for asynchronous loading of ACE)
    code.attachAceEditor(
      this.refs.editor.getNativeElement(),
      node[this.props.codeProperty],
      {
        language: language,
        fontSize: 15,
        // FIXME
        // This does not update when the editor state is changed (e.g editing turned from off to on)
        // Probably needs a custom event like `_onContentChanged` below
        readOnly: !this.context.controller.state.edit
      },
      function(editor) {
        // When editor has been created...

        // Additional options
        // ESC keypress
        editor.commands.addCommand({
          name: 'escape',
          bindKey: {win: 'Escape', mac: 'Escape'},
          exec: function(editor) {
            this.send('escape');
            editor.blur();
          }.bind(this),
          readOnly: true
        });

        editor.on('blur', this._onEditorBlur.bind(this));
        this.editor = editor;
      }.bind(this)
    );

    node.on(this.props.codeProperty + ':changed', this._onCodeChanged, this);
    if (this.props.languageProperty) node.on(this.props.languageProperty + ':changed', this._onLanguageChanged, this);
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
    var node = this.props.node;
    var codeProperty = this.codeProperty;
    var code = this.editor.getValue();
    if (code !== this.props.node[codeProperty]) {
      this.context.surface.transaction(function(tx) {
        tx.set([node.id, codeProperty], code);
      }, { source: this, skipSelection: true });
    }
  };

  /**
   * When the node's code changes, update the 
   * editor (if this wasn't the source of the update)
   */
  this._onCodeChanged = function(change, info) {
    var codeProperty = this.codeProperty;
    if (info.source !== this && this.editor) {
      this.editor.setValue(this.props.node[codeProperty], -1);
    }
  }

  /**
   * When the node's language changes, update the 
   * editor (if this wasn't the source of the update)
   */
  this._onLanguageChanged = function(change, info) {
    var languageProperty = this.languageProperty;
    if (info.source !== this && this.editor) {
      code.setAceEditorMode(this.editor, this.props.node[languageProperty]);
    }
  }

};

Component.extend(CodeEditorComponent);

CodeEditorComponent.fullWidth = true;

module.exports = CodeEditorComponent;
