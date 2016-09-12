'use strict';

var Tool = require('substance/ui/Tool');

/**
 * Tool for viewing info on the current Stencila component copy
 *
 * Displays the current copy and the number of users working on it (for collaborative
 * sessions). In future will probably allow for switching to a different copy.
 *
 * @class      CopyTool (name)
 */
function CopyTool () {
  CopyTool.super.apply(this, arguments);
}

CopyTool.Prototype = function () {
  var _super = CopyTool.super.prototype;

  this.getClassNames = function () {
    return _super.getClassNames.call(this) + ' se-copy-tool';
  };

  this.renderIcon = function ($$) {
    var el = $$('i');
    if (this.props.copy) {
      el.addClass('fa fa-users');
    } else {
      el.addClass('fa fa-user');
    }
    return el;
  };

  this.getTitle = function () {
    if (this.props.copy) {
      return 'You\'re working on the "' + this.props.copy.name + '" copy with ' + (this.props.copy.people - 1) + ' others';
    } else {
      return 'Master or local copy';
    }
  };
};

Tool.extend(CopyTool);

module.exports = CopyTool;

