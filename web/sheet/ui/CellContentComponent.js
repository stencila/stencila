'use strict';

var Component = require('substance/ui/Component');

function CellContentComponent() {
  CellContentComponent.super.apply(this, arguments);
}

CellContentComponent.Prototype = function() {

  this.didMount = function() {
    var node = this.props.node;
    this.doc = node.getDocument();
    this.doc.getEventProxy('path').connect(this, [node.id, 'displayMode'], this.rerender);
  };

  this.dispose = function() {
    this.doc.getEventProxy('path').disconnect(this);
  };
};

Component.extend(CellContentComponent);

module.exports = CellContentComponent;






