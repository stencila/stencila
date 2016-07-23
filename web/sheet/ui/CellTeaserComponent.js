'use strict';

var Component = require('substance-fe0ed/ui/Component');
var $$ = Component.$$;

function CellTeaserComponent() {
  CellTeaserComponent.super.apply(this, arguments);
}

CellTeaserComponent.Prototype = function() {

  this.render = function() {
    var cell = this.props.node;
    var el = $$('table').addClass('sc-cell-teaser');
    var tr = $$('tr');
    
    var prefix = cell.getPrefix();
    if (prefix) {
      tr.append(
        $$('td').addClass('se-name').text(prefix)
      );
    }

    var value = cell.value;
    var type = this.props.typeLabel || cell.valueType;
    if (value === undefined) {
      tr.append(
        $$('td').addClass('se-loading').text('Loading...')
      );
    } else if (type) {
      tr.append(
        $$('td').addClass('se-content-type').text(type)
      );
    }
    el.append(tr);

    return el;
  };

};

Component.extend(CellTeaserComponent);

module.exports = CellTeaserComponent;
