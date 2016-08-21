'use strict';

var Component = require('substance/ui/Component');
var TextPropertyComponent = require('substance/ui/TextPropertyComponent');

function MathCodeComponent() {
  MathCodeComponent.super.apply(this, arguments);
}

MathCodeComponent.Prototype = function() {

  var _super = MathCodeComponent.super.prototype;

  this.render = function($$) {
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
        $$(TextPropertyComponent, {
          path: [ node.id, 'source'],
          withoutBreak: true
        }),
        delim
      );
  };

};

Component.extend(MathCodeComponent);

module.exports = MathCodeComponent;
