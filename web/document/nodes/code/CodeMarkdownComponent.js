'use strict';

var AnnotationComponent = require('substance/ui/AnnotationComponent');

function CodeCodeComponent () {
  CodeCodeComponent.super.apply(this, arguments);
}

CodeCodeComponent.Prototype = function () {
  var _super = CodeCodeComponent.super.prototype;

  this.render = function ($$) {
    var el = _super.render.call(this, $$);
    return el;
  };
};

AnnotationComponent.extend(CodeCodeComponent);

module.exports = CodeCodeComponent;
