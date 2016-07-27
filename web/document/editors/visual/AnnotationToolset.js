'use strict';

var Component = require('substance/ui/Component');
var Tool = require('substance/ui/Tool');

var Toolset = require('../Toolset');

function AnnotationToolset() {
  Component.apply(this, arguments);
}

AnnotationToolset.Prototype = function() {

  this.render = function($$) {

    var el = $$('div').addClass('sc-toolset sc-annotation-toolset');

    var disabled = true;
    ['emphasis', 'strong', 'subscript', 'superscript', 'code', 'link'].forEach(function(name) {
      var tool = this.props.toolRegistry.get(name);
      var state = this._getCommandState(name);
      el.append(
        $$(tool.Class, state)
      );
      disabled = disabled && state.disabled;
    }.bind(this));

    if (disabled) el.addClass('sm-disabled');

    return el;
  };

};

Toolset.extend(AnnotationToolset);

module.exports = AnnotationToolset;
