'use strict';

var Component = require('substance/ui/Component');
var CellTeaserComponent = require('./CellTeaserComponent');
var $$ = Component.$$;

function ImageComponent() {
  ImageComponent.super.apply(this, arguments);
}

ImageComponent.Prototype = function() {

  this.render = function() {
    var el = $$('div').addClass('sc-object');
    el.addClass(this.props.displayMode);

    // Display cell teaser
    el.append($$(CellTeaserComponent, {node: this.props.node}));
    el.append(
      $$('img').attr('src', node.value)
    );
    return el;
  };
};

Component.extend(ImageComponent);

module.exports = ImageComponent;
