import Tool from 'substance/packages/tools/Tool'

/**
 * A class of `Tool` which instead of running a command
 * calls the `Blockset.changeType()` method to change the type
 * node
 *
 * @class      BlockTool (name)
 */
class BlockTool extends Tool {

  performAction () {
    if (this.props.active) {
      this.props.toolset.extendState({
        expanded: !this.props.toolset.state.expanded
      })
    } else {
      this.props.toolset.changeType(this.props.name)
    }
  }

}

export default BlockTool
