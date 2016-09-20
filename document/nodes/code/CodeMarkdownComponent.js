'use strict';

import AnnotationComponent from 'substance/ui/AnnotationComponent'

function CodeMarkdownComponent () {
  CodeMarkdownComponent.super.apply(this, arguments);
}

CodeMarkdownComponent.Prototype = function () {
  var _super = CodeMarkdownComponent.super.prototype;

  this.render = function ($$) {
    var el = _super.render.call(this, $$);
    return el;
  };
};

AnnotationComponent.extend(CodeMarkdownComponent);

module.exports = CodeMarkdownComponent;
