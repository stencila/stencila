import Tool from 'substance/packages/tools/Tool'

/**
 * Tool for viewing info on the current Stencila component copy
 *
 * Displays the current copy and the number of users working on it (for collaborative
 * sessions). In future will probably allow for switching to a different copy.
 *
 * @class      CopyTool (name)
 */
class CopyTool extends Tool {

  getClassNames () {
    return super.getClassNames.call(this) + ' se-copy-tool'
  }

  renderIcon ($$) {
    var el = $$('i')
    if (this.props.copy) {
      el.addClass('fa fa-users')
    } else {
      el.addClass('fa fa-user')
    }
    return el
  }

  getTitle () {
    if (this.props.copy) {
      return 'You\'re working on the "' + this.props.copy.name + '" copy with ' + (this.props.copy.people - 1) + ' others'
    } else {
      return 'Master or local copy'
    }
  }

}

export default CopyTool

