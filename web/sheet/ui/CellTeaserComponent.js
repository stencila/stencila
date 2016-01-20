'use strict';

var Component = require('substance/ui/Component');
var $$ = Component.$$;

function CellTeaserComponent() {
  CellTeaserComponent.super.apply(this, arguments);
}

CellTeaserComponent.Prototype = function() {

  this.render = function() {
    var cell = this.props.node;
    var el = $$('div').addClass('sc-cell-teaser');
    var name = cell.getName();
    el.append(
      $$('span').addClass('se-name').text(name),
      $$('span').addClass('se-content-type').text(cell.valueType)
    );
    return el;
  };
};

Component.extend(CellTeaserComponent);

module.exports = CellTeaserComponent;
