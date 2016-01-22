'use strict';

var Component = require('substance/ui/Component');
var CellComponent = require('./CellComponent');
var $$ = Component.$$;

/**
  Displays constant cells, such that don't start with '='.

  Possible values of content are:

  '10'
  '10.5'
  'Hello world'
  'Hell <strong>world</strong>'
*/

function ConstantComponent() {
  ConstantComponent.super.apply(this, arguments);
}

ConstantComponent.Prototype = function() {

  this.render = function() {
    var el = $$('div').addClass('sc-cell sc-constant');
    el.html(this.props.node.content);
    return el;
  };
};

CellComponent.extend(ConstantComponent);

module.exports = ConstantComponent;
