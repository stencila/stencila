import Tool from 'substance/packages/tools/Tool'

/**
 * Tool for changing the view (e.g. visual, code)
 *
 * @class      ViewTool (name)
 */
class ViewTool extends Tool {

  render ($$) {
    return $$('div').attr({
      'class': 'sc-tool sc-view-tool',
      'title': 'Toggle view. Current: ' + this.props.view
    }).append(
      $$('button').append(
        $$('i').addClass(this.props.view === 'code' ? 'fa fa-file-code-o' : 'fa fa-file-text-o')
      ).on('click', () => {
        this.send('view-toggle')
      })
    )
  }

}

export default ViewTool
