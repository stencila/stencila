'use strict';

var CodeEditorComponent = require('../../ui/CodeEditorComponent');

function CodeblockComponent (parent, props) {
  props.codeProperty = 'source';
  props.languageProperty = 'language';
  CodeblockComponent.super.apply(this, arguments);
}

CodeblockComponent.Prototype = function () {
  var _super = CodeblockComponent.super.prototype;

  this.render = function ($$) {
    return _super.render.call(this, $$)
      .addClass('sc-codeblock');
  };
};

CodeEditorComponent.extend(CodeblockComponent);

CodeblockComponent.fullWidth = true;

module.exports = CodeblockComponent;
