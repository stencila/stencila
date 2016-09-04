'use strict';

var HeadingComponentBase = require('substance/packages/heading/HeadingComponent');

/**
 * A component for `Heading` nodes
 *
 * Extends Substance `HeadingComponent` but add an event to rerender
 * on a change to the level
 *
 * @class      HeadingComponent (name)
 */
function HeadingComponent () {
  HeadingComponent.super.apply(this, arguments);
}

HeadingComponent.Prototype = function () {
  this.didMount = function () {
    this.props.node.on('level:changed', this.rerender, this);
  };

  this.dispose = function () {
    this.props.node.off(this);
  };
};

HeadingComponentBase.extend(HeadingComponent);

module.exports = HeadingComponent;
