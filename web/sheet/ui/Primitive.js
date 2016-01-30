'use strict';

var Component = require('substance/ui/Component');
var $$ = Component.$$;

function Privitive() {
  Privitive.super.apply(this, arguments);
}

Privitive.Prototype = function() {
  this.render = function() {
    var node = this.props.node;
    var el = $$('div').addClass('sc-primitive');
    
    // TODO: maybe introduce displaymode expanded to show source also
    el.addClass(node.displayMode);
    el.append(node.value);
    
    return el;
  };
};

Component.extend(Privitive);
module.exports = Privitive;
