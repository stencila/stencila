var Tool = require('substance/ui/Tool');

/**
 * A class of `Tool` which instead of running a command
 * calls the `Blockset.changeType()` method to change the type
 * node
 *
 * @class      BlockTool (name)
 */
function BlockTool () {
  BlockTool.super.apply(this, arguments);
}

BlockTool.Prototype = function () {
  this.performAction = function () {
    if (this.props.active) {
      this.props.toolset.extendState({
        expanded: !this.props.toolset.state.expanded
      });
    } else {
      this.props.toolset.changeType(this.props.name);
    }
  };
};

Tool.extend(BlockTool);

module.exports = BlockTool;
