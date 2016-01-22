'use strict';

var Component = require('substance/ui/Component');
var ExpressionCell = require('./ExpressionCell');
var CellTeaserComponent = require('./CellTeaserComponent');
var $$ = Component.$$;

/**
  Displays preformatted expression cells, e.g. where
  value type is 'matrix' or 'data.frame'
*/

function PreformattedCell() {
  PreformattedCell.super.apply(this, arguments);
}

ExpressionCell.extend(PreformattedCell);

module.exports = PreformattedCell;
