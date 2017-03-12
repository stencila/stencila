import BlockTool from '../../ui/BlockTool'

/**
 * A tool to edit `Default` nodes
 *
 * @class      DefaultTool (name)
 */
class DefaultTool extends BlockTool {

  render ($$) {
    var node = this.props.node
    return super.render.call(this, $$)
      .addClass('sc-default-tool')
      .append(
        $$('div')
          .ref('details')
          .addClass('se-details')
          .append(
            $$('button')
              .ref('edit')
              .addClass('se-edit')
              .attr('title', 'Edit')
              .append(
                $$('i')
                  .addClass('fa fa-pencil')
              )
              .on('click', function (event) {
                node.emit('edit:toggle')
              })
          )
      )
  }
}

export default DefaultTool
