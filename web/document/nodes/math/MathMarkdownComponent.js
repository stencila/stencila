'use strict';

var Component = require('substance/ui/Component');
var TextPropertyEditor = require('substance/ui/TextPropertyEditor');

function MathCodeComponent () {
  MathCodeComponent.super.apply(this, arguments);
}

MathCodeComponent.Prototype = function () {
  this.render = function ($$) {
    var node = this.props.node;

    var delim;
    if (node.language === 'asciimath') {
      delim = '|';
    } else {
      delim = '$';
    }

    return $$('span')
      .addClass('sc-math')
      .append(
        delim,
        $$(TextPropertyEditor, {
          path: [ node.id, 'source' ],
          withoutBreak: true
        }),
        delim
      );
  };
};

Component.extend(MathCodeComponent);

module.exports = MathCodeComponent;
