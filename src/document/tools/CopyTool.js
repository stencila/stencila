import { Tool } from 'substance'

/**
 * Tool for viewing info on the current Stencila component copy
 *
 * Displays the current copy and the number of users working on it (for collaborative
 * sessions). In future will probably allow for switching to a different copy.
 *
 * @class      CopyTool (name)
 */
class CopyTool extends Tool {

  render ($$) {
    return $$('div').attr({
      'class': 'sc-tool sc-copy-tool',
      'title': this.props.copy ? (
        'You\'re working on the "' + this.props.copy.name + '" copy with ' + (this.props.copy.people - 1) + ' others'
        ) : 'Master or local copy'
    }).append(
      $$('button').append(
        $$('i').addClass(this.props.copy ? 'fa fa-users' : 'fa fa-user')
      )
    )
  }

}

export default CopyTool

