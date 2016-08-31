'use strict';

var HeadingComponent = require('./HeadingComponent');

function HeadingMarkdownComponent () {

  HeadingMarkdownComponent.super.apply(this, arguments);

}

HeadingMarkdownComponent.Prototype = function () {

  var _super = HeadingMarkdownComponent.super.prototype;

  this.render = function ($$) {

    var node = this.props.node;
    return _super.render.call(this, $$)
      .insertAt(0,
        $$('span')
          .ref('level')
          .text(Array(node.level + 1).join('#') + ' ')
      );

  };

};

HeadingComponent.extend(HeadingMarkdownComponent);

module.exports = HeadingMarkdownComponent;
