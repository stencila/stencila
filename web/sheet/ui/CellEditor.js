'use strict';

var Component = require('substance/ui/Component');
var $$ = Component.$$;
var SnippetComponent = require('./SnippetComponent');

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
      $$('input')
        .attr('value', this.props.content)
        .on('keydown', this.onKeydown)
        .on('keypress', this.onKeypress)
        .on('click', this.onClick)
        .on('input', this.onChange)
        .ref('editor')
    );
    if (this.state.snippet) {
      el.append($$(SnippetComponent, {
        snippet: require('../testSnippet'),
        paramIndex: this.state.paramIndex
      }));
    }
    return el;
  };

  this.didMount = function() {
    var el = this._getTextArea();
    el.focus();
    if (this.props.select === "all") {
      el.select();
    } else if (this.props.select === "last") {
      var l = this.props.content.length;
      el.setSelectionRange(l, l);
    }
  };

  this.getContent = function() {
    return this._getTextArea().value;
  };

  this._getTextArea = function() {
    return this.el.querySelector('input');
  };

  this.onKeydown = function(event) {
    // console.log('CellEditor.onKeydown()', 'keyCode=', event.keyCode, event);
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
    // console.log('CellEditor.onKeypress()', 'keyCode=', event.keyCode);
    var handled = false;
    if (handled) {
      event.stopPropagation();
      event.preventDefault();
    }
  };

  this.onClick = function() {
    this._detectSnippet();
  };

  this.onChange = function() {
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
          var snippet = lastMatch[1];
          var startPos = lastMatch.index+1;
          var argsPos = startPos + lastMatch[0].length;
          var currentArg = this._detectCurrentArg(source.slice(argsPos));
          var newState = {
            snippet: snippet,
            paramIndex: currentArg.argIdx
          };
          // console.log('DETECTED SNIPPET', newState);
          this.setState(newState);
        } else if (this.state.snippet) {
          this.extendState({
            snippet: false
          });
        }
      }
    }.bind(this));
  };

  this._detectCurrentArg = function(str) {
    // on each ',' increase counter
    // on '(' skip content to allow for nested expressions
    var argIdx = 0;
    var stackCount = 0;
    var skip = false;
    for(var pos = 0; pos < str.length; pos++) {
      var c = str[pos];
      if (skip) {
        if (c === '(') {
          stackCount++;
        } else if (c === ')') {
          stackCount--;
        }
        if (stackCount === 0) {
          skip = false;
        }
      }
      else if (c === ',') {
        argIdx++;
      }
      else if (c === '(') {
        stackCount++;
        skip = true;
      }
    }
    return {
      argIdx: argIdx
    };
  };

};

Component.extend(CellEditor);

module.exports = CellEditor;
