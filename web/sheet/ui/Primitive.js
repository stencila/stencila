'use strict';

var Component = require('substance/ui/Component');
var $$ = Component.$$;

function Primitive() {
  Primitive.super.apply(this, arguments);
}

Primitive.Prototype = function() {
  this.render = function() {
    var cell = this.props.node;
    var el = $$('table').addClass('sc-primitive');
    var tr = $$('tr');
    var name = cell.getName() || '';
    name += ' =';
    tr.append(
      $$('td').addClass('se-name').text(name)
    );
    var value = cell.value;
    if (value === undefined) {
      value = 'Loading';
    }
    tr.append($$('td').addClass('se-value').text(value));

    el.append(tr);
    return el;
  };
};

Component.extend(Primitive);

Primitive.static.displayModes = ['expanded'];

module.exports = Primitive;
