'use strict';

var CodeEditorComponent = require('../../ui/CodeEditorComponent');

function ExecuteComponent() {
  ExecuteComponent.super.apply(this, arguments);
}

ExecuteComponent.Prototype = function() {

  var _super = ExecuteComponent.super.prototype;

  this.render = function($$) {
    var el = _super.render.call(this, $$);
    return el.addClass('sc-execute');
  };

};

CodeEditorComponent.extend(ExecuteComponent);

module.exports = ExecuteComponent;
