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
  'Hello <strong>world</strong>'
*/

function ConstantComponent() {
  ConstantComponent.super.apply(this, arguments);
}

ConstantComponent.Prototype = function() {
  var _super = Object.getPrototypeOf(this);
  
  this.render = function() {
    var el = _super.render.call(this);
    el.addClass('sc-constant-cell');
    if (!this.isEditing()) {
      el.append(this.props.node.content);
    }
    return el;
  };
};

CellComponent.extend(ConstantComponent);

module.exports = ConstantComponent;
