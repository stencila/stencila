'use strict';

var Panel = require('substance/ui/Panel');
var Component = require('substance/ui/Component');
var Panel = require('substance/ui/Panel');
var DialogHeader = require('substance/ui/DialogHeader');
var $$ = Component.$$;

// List existing bib items
// -----------------

function EditSourcePanel() {
  Component.apply(this, arguments);

  var controller = this.getController();
  controller.connect(this, {
    'document:saved': this.refresh
  });
}

EditSourcePanel.Prototype = function() {

  this.dispose = function() {
    var controller = this.getController();
    controller.disconnect(this);
    this.editor.destroy();
  };

  this.didMount = function() {
    this.initAce();
  };

  // Rerender to show the error message and also update Ace
  this.refresh = function() {
    var node = this.getNode();
    var errorContainer = this.refs.errorContainer;
    errorContainer.empty();
    if (node.error) {
      errorContainer.append($$('div').addClass('se-error').append(node.error));  
    }
  };

  this.initAce = function() {
    var editor = this.editor = window.ace.edit('ace_editor');
    editor.getSession().setMode('ace/mode/r');
    editor.setTheme("ace/theme/monokai");

    editor.setFontSize(14);
    editor.setShowPrintMargin(false);
    // Set the maximum number of lines for the code. When the number
    // of lines exceeds this number a vertical scroll bar appears on the right
    editor.setOption("minLines",5);
    editor.setOption("maxLines",100000);
    // Prevent warning message
    editor.$blockScrolling = Infinity;
    // Set indented wrapped lines
    editor.setOptions({
      wrap: true,
      indentedSoftWrap: true,
    });

    var node = this.getNode();
    editor.setValue(node.source,1);

    editor.on('change', function() {
      node.setSource(editor.getValue());
    });
  };

  this.handleCancel = function(e) {
    e.preventDefault();
    this.send('switchContext', 'toc');
  };

  this.getNode = function(){
    var doc = this.getDocument();
    var node = doc.get(this.props.nodeId);
    return node;
  };

  this.render = function() {
    var node = this.getNode();

    var panelEl = $$(Panel).ref('panelEl');
    var errorContainerEl = $$('div').ref('errorContainer');
    if (node.error) {
      errorContainerEl.append(
        $$('div').addClass('se-error').append(node.error)
      );
    }

    panelEl.append(errorContainerEl);
    panelEl.append(
      $$('div').attr('id','ace_editor')
    );

    return $$('div').addClass('sc-edit-source-panel').append(
      $$(DialogHeader, {label: 'Edit Source'}),
      panelEl      
    );
  };
};

Component.extend(EditSourcePanel);
module.exports = EditSourcePanel;
