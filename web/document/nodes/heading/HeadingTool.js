'use strict';

var BlockTool = require('../../ui/BlockTool');

/**
 * A tool to edit heading (change the heading level)
 *
 * @class      HeadingTool (name)
 */
function HeadingTool() {
  HeadingTool.super.apply(this, arguments);
}

HeadingTool.Prototype = function() {

  var _super = HeadingTool.super.prototype;

  this.render = function($$) {
    var node = this.props.node;
    return _super.render.call(this, $$)
      .addClass('sc-heading-tool')
      .append(
        $$('div')
          .ref('details')
          .addClass('se-details')
          .append(
            $$('span')
              .ref('level')
              .text(''+node.level)
              .on('click', function(event){
                event.stopPropagation();
                event.preventDefault();
                // FIXME
                // 1. On click the tool disappears (but not if no transaction)
                // 2. Although the transaction seems to work, the
                //    heading is not rerendered (click on it again and level is updated)
                var session = this.context.documentSession;
                session.transaction(function(tx) {
                  tx.set([node.id, 'level'], node.level==6 ? 1 : node.level+1);
                });
              }.bind(this))
          )
      );
  };

};

BlockTool.extend(HeadingTool);

module.exports = HeadingTool;
