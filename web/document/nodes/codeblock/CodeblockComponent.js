'use strict';

var CodeEditorComponent = require('../../ui/CodeEditorComponent');


function CodeblockComponent() {
  CodeblockComponent.super.apply(this, arguments);
}

CodeblockComponent.Prototype = function() {

  var _super = CodeblockComponent.super.prototype;

  this.render = function($$) {
    var el = _super.render.call(this, $$);
    el.addClass('sc-codeblock');
    return el;
  };

};

CodeEditorComponent.extend(CodeblockComponent);

CodeblockComponent.fullWidth = true;

module.exports = CodeblockComponent;
