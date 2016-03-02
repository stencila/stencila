'use strict';

var Component = require('substance/ui/Component');
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
    var el = $$('table').addClass('sc-primitive');
    var tr = $$('tr');

    var prefix = cell.getPrefix();
    tr.append(
      $$('td').addClass('se-name').text(prefix)
    );

    var value = cell.value;
    var className = 'se-value';
    if (value === undefined) {
      value = 'Loading';
      className = 'se-loading'
    }
    tr.append(
      $$('td').addClass(className).text(value)
    );

    el.append(tr);
    return el;
  };

};

Component.extend(Primitive);

Primitive.static.displayModes = ['expanded'];

module.exports = Primitive;
