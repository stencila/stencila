'use strict';

var Component = require('substance/ui/Component');
var CellContentComponent = require('./CellContentComponent');
var $$ = Component.$$;

function TextComponent() {
  TextComponent.super.apply(this, arguments);
}

TextComponent.Prototype = function() {

  this.render = function() {
    var el = $$('div').addClass('sc-cell-content sc-text');
    el.text(this.props.node.value);
    return el;
  };
};

CellContentComponent.extend(TextComponent);

module.exports = TextComponent;
