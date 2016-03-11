'use strict';

var Component = require('substance/ui/Component');
var CellTeaserComponent = require('./CellTeaserComponent');
var $$ = Component.$$;

function Error() {
  Error.super.apply(this, arguments);
}

Error.Prototype = function() {
  this.render = function() {
    var node = this.props.node;
    var el = $$('div').addClass('sc-error');
    
    el.addClass(node.getDisplayClass());

    // Display cell teaser
    el.append($$(CellTeaserComponent, {node: node}));

    el.append(
      $$('div').addClass('se-error-message').append(node.value)
    );
    return el;
  };
};

Component.extend(Error);

module.exports = Error;
