import AnnotationMacro from '../../ui/AnnotationMacro'

/**
 * A macro for creating `Code` nodes
 *
 * Uses Markdown syntax of enclosing backticks.
 *
 * @class      CodeMacro (name)
 */
class CodeMacro extends AnnotationMacro {

  get regex () {
    return /`([^`]+)`/
  }

  createNodeData (match) {
    return {
      type: 'code',
      text: match[1]
    }
  }

}

export default CodeMacro
