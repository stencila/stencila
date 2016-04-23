'use strict';

var oo = require('substance/util/oo');
var Component = require('substance/ui/Component');
var $$ = Component.$$;

var StencilNodeComponent = require('../../StencilNodeComponent');

function StencilCodeblockComponent() {
  StencilNodeComponent.apply(this, arguments);
}

StencilCodeblockComponent.Prototype = function() {

  this.render = function() {
    var node = this.props.node;

    var el = $$('div')
      .attr("data-id", node.id)
      .attr("contenteditable", false);
  
    this.editorId = Math.random().toString(36).replace(/[^a-z]+/g, '');
    el.append(
      $$('pre')
        .addClass('se-codeblock-source')
        .attr('id',this.editorId)
        .attr("contenteditable", false)
        .text(node.source)
    );

    return el;
  };

  this.didMount = function() {
    var node = this.props.node;
    if (window.ace) {
      var editor = this.editor = window.ace.edit(this.editorId);

      // Convert language tag to AC mode if necessary
      var mode = {
        '': 'text',
        'js':   'javascript',
        'py':   'python',
      }[node.lang] || node.lang;
      editor.getSession().setMode('ace/mode/'+mode);
      
      editor.setTheme("ace/theme/monokai");
      editor.setFontSize(13);
      editor.setShowPrintMargin(false);
      // Add padding before first and after last lines
      editor.renderer.setScrollMargin(5,5,0,0);
      // Set the maximum number of lines for the code. When the number
      // of lines exceeds this number a vertical scroll bar appears on the right
      editor.setOption("minLines",1);
      editor.setOption("maxLines",100);
      // Prevent warning message
      editor.$blockScrolling = Infinity;
      // Make readonly as per https://github.com/ajaxorg/ace/issues/266#issuecomment-16367687
      editor.setOptions({
        readOnly: true,
        highlightActiveLine: false,
        highlightGutterLine: false,

        wrap: true,
        indentedSoftWrap: true,
      });
      editor.renderer.$cursorLayer.element.style.opacity = 0;
      editor.textInput.getElement().disabled = true;
      editor.commands.commmandKeyBinding = {};

      editor.setValue(node.source,1);
    }
  };

};

oo.inherit(StencilCodeblockComponent, StencilNodeComponent);

module.exports = StencilCodeblockComponent;
