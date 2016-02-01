'use strict';

var Component = require('substance/ui/Component');
var $$ = Component.$$;

function Primitive() {
  Primitive.super.apply(this, arguments);
}

Primitive.Prototype = function() {
  this.render = function() {
    var node = this.props.node;
    var el = $$('div').addClass('sc-primitive');

    // TODO: maybe introduce displaymode expanded to show source also
    el.addClass(node.displayMode);
    el.append(node.value);

    return el;
  };
};

Component.extend(Primitive);
module.exports = Primitive;
