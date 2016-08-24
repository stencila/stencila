'use strict';

var Tool = require('substance/ui/Tool');

/**
 * Tool for changing the view (e.g. visual, code)
 * 
 * @class      ViewTool (name)
 */
function ViewTool() {
  ViewTool.super.apply(this, arguments);
}

ViewTool.Prototype = function() {

  var _super = ViewTool.super.prototype;

  this.getClassNames = function() {
    return _super.getClassNames.call(this) + ' se-view-tool';
  }

  this.renderIcon = function($$) {
    var el = $$('i');
    if (this.props.view === 'code') {
      el.addClass('fa fa-file-code-o');
    } else {
      el.addClass('fa fa-file-text-o');
    }
    return el;
  };

  this.getTitle = function() {
    return 'Toggle view. Current: ' + this.props.view;
  };

  this.onClick = function() {
    this.send('view-toggle');
  }

};

Tool.extend(ViewTool);

module.exports = ViewTool;

