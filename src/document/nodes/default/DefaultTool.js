import BlockTool from '../../ui/BlockTool'

/**
 * A tool to edit `Default` nodes
 *
 * @class      DefaultTool (name)
 */
class DefaultTool extends BlockTool {

  render ($$) {
    const el = super.render($$)
    el.addClass('sc-default-tool')

    const detail = $$('div').ref('details').addClass('se-details')
    detail.append(
      $$('button').ref('edit')
        .addClass('se-edit')
        .attr('title', 'Edit')
        .append(
          $$('i')
            .addClass('fa fa-pencil')
        )
        .on('click', this.onEditClick)
    )
    el.append(detail)
    return el
  }

  onEditClick(event) {
    event.preventDefault()
    event.stopPropagation()
    const node = this.props.node
    node.emit('edit:toggle')
  }
}

export default DefaultTool
