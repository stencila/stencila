'use strict';

var Component = require('substance-fe0ed/ui/Component');
var $$ = Component.$$;

/**
 * Used for displaying cells which are have `html` as their
 * value `type`.
 */
function HTMLCellComponent() {
  HTMLCellComponent.super.apply(this, arguments);
}

HTMLCellComponent.Prototype = function() {

  this.render = function() {
    var cell = this.props.node;
    var el = $$('div').addClass('sc-html-cell');

    var value = cell.value;
    var className = '';
    if (value === undefined) {
      value = 'Loading';
      className = 'sm-loading'
    }
    el.addClass(className).html(value);

    if(window.MathJax && window.MathJax.Hub) {
      MathJax.Hub.Queue(["Rerender", MathJax.Hub, cell.id]);
    }

    return el;
  };

};

Component.extend(HTMLCellComponent);

module.exports = HTMLCellComponent;
