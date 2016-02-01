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
    var name = cell.getName() || '';
    var tr = $$('tr');
    if (name) {
      tr.append(
        $$('td').addClass('se-name').text(name)
      );
    }
    if (cell.value === undefined) {
      tr.append($$('td').addClass('se-loading').text('Loading...'));
    } else if (cell.valueType) {
      tr.append(
        $$('td').addClass('se-content-type').text(cell.valueType)
      );
    }
    el.append(tr);
    return el;
  };

};

Component.extend(CellTeaserComponent);

module.exports = CellTeaserComponent;
