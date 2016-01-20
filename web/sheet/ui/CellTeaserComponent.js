'use strict';

var Component = require('substance/ui/Component');
var $$ = Component.$$;

function CellTeaserComponent() {
  CellTeaserComponent.super.apply(this, arguments);
}

CellTeaserComponent.Prototype = function() {

  this.render = function() {
    var cell = this.props.node;
    var el = $$('table').addClass('sc-cell-teaser');
    var name = cell.getName();

    el.append(
      $$('tr').append(
        $$('td').addClass('se-name').text(name),
        $$('td').addClass('se-content-type').text(cell.valueType)
      )
    );
    return el;
  };
};

Component.extend(CellTeaserComponent);

module.exports = CellTeaserComponent;
