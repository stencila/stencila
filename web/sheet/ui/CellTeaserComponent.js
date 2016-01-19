'use strict';

var Component = require('substance/ui/Component');
var $$ = Component.$$;

function CellTeaserComponent() {
  CellTeaserComponent.super.apply(this, arguments);
}

CellTeaserComponent.Prototype = function() {

  this.render = function() {
    var node = this.props.node;
    var el = $$('div').addClass('sc-cell-teaser');
    
    el.append(
      $$('span').addClass('se-name').text(node),
      $$('span').addClass('se-content-type').text(node.getContentType())
    );
  };
};

Component.extend(CellTeaserComponent);

module.exports = CellTeaserComponent;
