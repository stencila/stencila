'use strict';

var Component = require('substance/ui/Component');
var $$ = Component.$$;

/**
  Displays constant cells, such that don't start with '='.

  Possible values of content are:

  '10'
  '10.5'
  'Hello world'
  'Hello <strong>world</strong>'
*/

function Constant() {
  Constant.super.apply(this, arguments);
}

Constant.Prototype = function() {
  this.render = function() {
    var el = $$('div').addClass('sc-constant');
    el.append(this.props.node.content);
    return el;
  };
};

Component.extend(Constant);

Constant.static.displayModes = ['expanded'];

module.exports = Constant;
