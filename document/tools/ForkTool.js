'use strict'

import Tool from 'substance/packages/tools/Tool'

function ForkTool () {
  ForkTool.super.apply(this, arguments)
}

ForkTool.Prototype = function () {
  this.getTitle = function () {
    return 'Create a fork of this document; not yet implemented :('
  }
}

Tool.extend(ForkTool)

export default ForkTool

