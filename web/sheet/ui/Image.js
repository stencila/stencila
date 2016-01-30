'use strict';

var Component = require('substance/ui/Component');
var CellTeaserComponent = require('./CellTeaserComponent');
var $$ = Component.$$;

function Image() {
  Image.super.apply(this, arguments);
}

Image.Prototype = function() {

  this.render = function() {
    var node = this.props.node;
    // Using .sc-sheet-image instead of .sc-image so we don't have style
    // clashes with native Substance Image
    var el = $$('div').addClass('sc-sheet-image');
    el.addClass(node.displayMode);
    
    el.append($$(CellTeaserComponent, {node: node}));

    if (node.displayMode != 'clipped') {
      el.append(
        $$('img').attr('src', node.value)
      );
    }
    return el;
  };
};

Component.extend(Image);

module.exports = Image;
