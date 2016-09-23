'use strict'

import AnnotationMacro from '../../ui/AnnotationMacro'

/**
 * A macro for creating `Code` nodes
 *
 * Uses Markdown syntax of enclosing backticks.
 *
 * @class      CodeMacro (name)
 */
function CodeMacro () {
};

CodeMacro.Prototype = function () {
  this.appliesTo = []

  this.regex = /`([^`]+)`/

  this.createNodeData = function (match) {
    return {
      type: 'code',
      text: match[1]
    }
  }
}

AnnotationMacro.extend(CodeMacro)

export default CodeMacro
