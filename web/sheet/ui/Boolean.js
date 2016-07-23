'use strict';

var Component = require('substance-fe0ed/ui/Component');
var $$ = Component.$$;
var Icon = require('substance-fe0ed/ui/FontAwesomeIcon');

/**
 * Displays cells which have a boolean (true/false)
 * value type
 */
function Boolean() {
  Boolean.super.apply(this, arguments);
}

Boolean.Prototype = function() {

  this.render = function() {
    var cell = this.props.node;

    var el = $$('div').addClass('sc-boolean');

    var prefix = cell.getPrefix();
    el.append(
      $$('span').addClass('se-prefix').text(prefix)
    );

    // Using lowercase below allows for alternative string representations
    // in different languages eg. TRUE in R, True in Python, true in Javascript
    var value = cell.value;
    var icon;
    var className;
    if (value === undefined) {
      icon = 'spinner';
      className = 'sm-loading';
    }
    else  if (value.toLowerCase()=='true') {
      icon = 'check';
      className = 'sm-true';
    }
    else if (value.toLowerCase()=='false') {
      icon = 'times';
      className = 'sm-false';
    }
    el.append(
      $$(Icon, {icon: 'fa-'+ icon}).addClass('se-value ' + className)
    );

    return el;
  };

};

Component.extend(Boolean);

module.exports = Boolean;
