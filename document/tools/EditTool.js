'use strict'

import Tool from 'substance/packages/tools/Tool'

/**
 * Tool for toggling edit mode for a Stencila Document
 * `VisualEditor`
 *
 * @class      EditTool (name)
 */
function EditTool () {
  EditTool.super.apply(this, arguments)
}

EditTool.Prototype = function () {
  this.getTitle = function () {
    if (this.props.active) return 'Turn off editing'
    else return 'Turn on editing'
  }

  this.renderIcon = function ($$) {
    return $$('i').addClass('fa fa-pencil')
  }

  this.onClick = function () {
    this.send('edit-toggle')
  }
}

Tool.extend(EditTool)

module.exports = EditTool

