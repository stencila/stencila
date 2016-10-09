import Tool from 'substance/packages/tools/Tool'

/**
 * Tool for changing the view (e.g. visual, code)
 *
 * @class      ViewTool (name)
 */
class ViewTool extends Tool {

  getClassNames () {
    return super.getClassNames.call(this) + ' se-view-tool'
  }

  renderIcon ($$) {
    var el = $$('i')
    if (this.props.view === 'code') {
      el.addClass('fa fa-file-code-o')
    } else {
      el.addClass('fa fa-file-text-o')
    }
    return el
  }

  getTitle () {
    return 'Toggle view. Current: ' + this.props.view
  }

  onClick () {
    this.send('view-toggle')
  }

}

export default ViewTool
