'use strict';

var oo = require('substance/util/oo');
var Panel = require('substance/ui/Panel');
var Component = require('substance/ui/Component');
var Icon = require('substance/ui/FontAwesomeIcon');
var $$ = Component.$$;

function CilaPanel() {
  Panel.apply(this, arguments);

  var doc = this.getDocument();
  doc.connect(this, {
    'document:changed': this.handleDocumentChanged
  });
}

CilaPanel.Prototype = function() {

  this.dispose = function() {
    var controller = this.getController();
    controller.disconnect(this);
    this.editor.destroy();
  };

  this.render = function() {
    var el = $$('div')
      .addClass('sc-cila-panel panel')
      .append(
        $$('div').attr('id','ace_editor')
      );
    return el;
  };

  this.didMount = function() {
    this.initAce();
  };

  this.initAce = function() {
    var editor = this.editor = ace.edit('ace_editor');
    editor.getSession().setMode('ace/mode/cila');
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

    this.update();

    editor.on('change', function() {
      console.log('changed cila');
    });
  };

  this.handleDocumentChanged = function(change) {
    console.log(change);
  };

  this.update = function(){
    var doc = this.getDocument();
    doc.getCila(function(cila){
      this.editor.setValue(cila,1);
    }.bind(this));
  };

};

oo.inherit(CilaPanel, Panel);

module.exports = CilaPanel;
