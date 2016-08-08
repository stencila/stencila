'use strict';

var LinkComponentBase = require('substance/packages/link/LinkComponent');

/**
 * A component for `Link` nodes
 * 
 * Extends Substance `LinkComponent` but add an event to rerender 
 * on a change to the `url` property
 *
 * @class      LinkComponent (name)
 */
function LinkComponent() {
  LinkComponent.super.apply(this, arguments);
}

LinkComponent.Prototype = function() {

  this.didMount = function() {
    this.props.node.on('url:changed', this.rerender, this);
  };

  this.dispose = function() {
    this.props.node.off(this);
  };

};

LinkComponentBase.extend(LinkComponent);

module.exports = LinkComponent;
