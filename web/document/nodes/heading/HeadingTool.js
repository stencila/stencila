'use strict';

var map = require('substance/node_modules/lodash/map');

var BlockTool = require('../../ui/BlockTool');

/**
 * A tool to edit `Heading` nodes (change the heading level)
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
            $$('select')
              .ref('level')
              .append(map([1,2,3,4,5,6], function(level){
                var option = $$('option')
                  .val(level)
                  .html(level);
                if (level == node.level) option.attr('selected', true);
                return option;
              }))
              .on('change', function(event){
                var session = this.context.documentSession;
                session.transaction(function(tx) {
                  tx.set([node.id, 'level'], parseInt(event.target.value));
                });
              }.bind(this))
          )
      );
  };

};

BlockTool.extend(HeadingTool);

module.exports = HeadingTool;
