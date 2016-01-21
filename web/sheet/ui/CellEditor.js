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
    var el = $$('div');
    el.append(
      $$('textarea')
        .text(this.props.content)
        .on('keydown', this.onKeydown)
        .on('keypress', this.onKeypress)
        .on('click', this.onClick)
        .on('input', this.onChange)
        .ref('editor')
    );
    if (this.state.snippet) {
      el.append(
        $$('div').ref(this.state.snippet)
          .text('DESCRIPTION OF SNIPPET' + this.state.snippet)
      );
    }
    return el;
  };

  this.didMount = function() {
    var el = this._getTextArea();
    el.focus();
    el.select();
  };

  this.getContent = function() {
    return this._getTextArea().value;
  };

  this._getTextArea = function() {
    return this.refs.editor.el;
  };

  this.onKeydown = function(event) {
    console.log('CellEditor.onKeydown()', 'keyCode=', event.keyCode, event);
    var handled = false;
    // ENTER
    if (event.keyCode === 13) {
      if (!event.ctrlKey && !event.shiftKey) {
        this.send('commitCellChange', this.getContent(), 'enter');
        handled = true;
      }
    } else if (event.keyCode === 27) {
      this.send('discardCellChange');
      handled = true;
    } else if (event.keyCode >= 37 && event.keyCode <= 40) {
      this._detectSnippet();
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

  this.onClick = function(event) {
    this._detectSnippet();
  };

  this.onChange = function(event) {
    this._detectSnippet();
  };

  // SNIPPETS SPIKE

  var _SNIPPETS = ['sum', 'mean'];
  var _SNIPPET_RE_STR = '\\b(' + _SNIPPETS.join('|') + ')[(]';

  this._detectSnippet = function() {
    setTimeout(function() {
      var el = this._getTextArea();
      var source = el.value;
      var pos = el.selectionStart;
      // only if collapsed
      if (pos === el.selectionEnd) {
        source = source.slice(0, pos);
        var re = new RegExp(_SNIPPET_RE_STR, 'gi');
        var lastMatch, match;
        while ( (match = re.exec(source)) ) {
          lastMatch = match;
        }
        if (lastMatch) {
          // console.log('DETECTED SNIPPET', lastMatch[1], lastMatch);
          this.setState({
            snippet: lastMatch[1],
            startPos: lastMatch.index+1,
            argsStartPos: lastMatch.index+lastMatch[0].length,
          });
        } else if (this.state.snippet) {
          this.extendState({
            snippet: false
          });
        }
      }
    }.bind(this));
  };

  this._updateSnippet = function() {
    setTimeout(function() {
      var el = this._getTextArea();
      var pos = el.selectionStart;
      // TODO: check that we are inside the snippets arguments
      if (pos < this.state.argsStartPos) {
        this.setState({});
      }
    }.bind(this));
  };

};

Component.extend(CellEditor);

module.exports = CellEditor;
