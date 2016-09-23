'use strict';

import CodeEditorComponent from '../../ui/CodeEditorComponent'

function ExecuteComponent (parent, props) {
  props.codeProperty = 'source';
  props.languageProperty = 'language';
  ExecuteComponent.super.call(this, parent, props);
}

ExecuteComponent.Prototype = function () {
  var _super = ExecuteComponent.super.prototype;

  this.render = function ($$) {
    return _super.render.call(this, $$)
      .addClass('sc-execute');
  };
};

CodeEditorComponent.extend(ExecuteComponent);

module.exports = ExecuteComponent;
