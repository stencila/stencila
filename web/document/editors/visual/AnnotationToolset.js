'use strict';

var Component = require('substance/ui/Component');
var Tool = require('substance/ui/Tool');
var documentHelpers = require('substance/model/documentHelpers');


var Toolset = require('../Toolset');

function AnnotationToolset() {
  Component.apply(this, arguments);
}

AnnotationToolset.Prototype = function() {

  this.render = function($$) {

    var el = $$('div').addClass('sc-toolset sc-annotation-toolset');

    // This should only appear when there is a user text selection or when the cursor
    // is on an existing annotation
    var session = this.context.documentSession;
    var text = documentHelpers.getTextForSelection(session.getDocument(), session.getSelection());
    if (!text.length) return el;

    var disabled = true;
    ['emphasis', 'strong', 'subscript', 'superscript', 'code', 'link', 'print'].forEach(function(name) {
      var tool = this.props.toolRegistry.get(name);
      var state = this._getCommandState(name);
      el.append(
        $$(tool.Class, state).ref(name)
      );
      disabled = disabled && state.disabled;
    }.bind(this));

    if (disabled) el.addClass('sm-disabled');

    return el;
  };

};

Toolset.extend(AnnotationToolset);

module.exports = AnnotationToolset;
