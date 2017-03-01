import { Tool } from 'substance'

/**
 * Tool for toggling edit mode for a Stencila Document
 * `VisualEditor`
 *
 * @class      EditTool (name)
 */
class EditTool extends Tool {

  getTitle () {
    if (this.props.active) return 'Turn off editing'
    else return 'Turn on editing'
  }

  renderIcon ($$) {
    return $$('i').addClass('fa fa-pencil')
  }

  onClick () {
    this.send('edit-toggle')
  }

}

export default EditTool

