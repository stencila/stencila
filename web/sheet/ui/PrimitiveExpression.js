'use strict';

var Component = require('substance-fe0ed/ui/Component');
var $$ = Component.$$;

/**
 * Used for displaying cells which are not constant (i.e. `kind` != 'lit') and 
 * have a `type` that is a primitive (e.g. number, integer, string).
 */
function Primitive() {
  Primitive.super.apply(this, arguments);
}

Primitive.Prototype = function() {

  this.render = function() {
    var cell = this.props.node;

    var el = $$('div').addClass('sc-primitive');

    var prefix = cell.getPrefix();
    el.append(
      $$('span').addClass('se-name').text(prefix)
    );

    var value = cell.value;
    var className = 'se-value';
    if (value === undefined) {
      value = 'Loading';
      className = 'se-loading'
    }
    el.append(
      $$('span').addClass(className).text(value)
    );

    return el;
  };

};

Component.extend(Primitive);

module.exports = Primitive;
