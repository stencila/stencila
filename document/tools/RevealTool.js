'use strict';

import Tool from 'substance/packages/tools/Tool'

/**
 * Tool for toggling the reveal mode of a
 * Stencila Document `VisualEditor`
 *
 * @class      RevealTool (name)
 */
function RevealTool () {
  RevealTool.super.apply(this, arguments);
}

RevealTool.Prototype = function () {
  this.getTitle = function () {
    if (this.props.active) return 'Don\'t show computations and comments';
    else return 'Show computations and comments';
  };

  this.onClick = function () {
    this.send('reveal-toggle');
  };
};

Tool.extend(RevealTool);

module.exports = RevealTool;

