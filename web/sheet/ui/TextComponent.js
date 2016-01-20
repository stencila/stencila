'use strict';

var Component = require('substance/ui/Component');
var CellTeaserComponent = require('./CellTeaserComponent');
var $$ = Component.$$;

function TextComponent() {
  TextComponent.super.apply(this, arguments);
}

TextComponent.Prototype = function() {

  this.render = function() {
    var el = $$('div').addClass('sc-text');
    el.text(this.props.node.content);
    return el;
  };
};

Component.extend(TextComponent);

module.exports = TextComponent;
