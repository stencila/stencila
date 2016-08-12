'use strict';

var Tool = require('substance/ui/Tool');

/**
 * Tool for viewing info on the current Stencila jam session
 * 
 * @class      JamTool (name)
 */
function JamTool() {
  JamTool.super.apply(this, arguments);
}

JamTool.Prototype = function() {

  var _super = JamTool.super.prototype;

  this.getClassNames = function() {
    return _super.getClassNames.call(this) + ' se-jam-tool';
  }

  this.renderIcon = function($$) {
    var el = $$('i');
    if (this.props.jam) {
      el.addClass('fa fa-users');
    } else {
      el.addClass('fa fa-user');
    }
    return el;
  };

  this.getTitle = function() {
    if (this.props.jam) {
      return 'You\'re in the "' + this.props.jam.name + '" jam with ' + (this.props.jam.people - 1) + ' others';
    } else {
      return 'This is a solo session';
    }
  };

};

Tool.extend(JamTool);

module.exports = JamTool;

